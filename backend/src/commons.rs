use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const COMMONS_API_URL: &str = "https://commons.wikimedia.org/w/api.php";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LicensedPhoto {
    pub url: String,
    pub author: String,
    pub license: String,
    pub license_url: String,
    pub attribution: String,
    pub source_url: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedPhoto {
    pub requested_title: String,
    pub photo: LicensedPhoto,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PhotoResolution {
    pub accepted: Vec<ResolvedPhoto>,
    pub rejected: usize,
}

pub async fn resolve_commons(
    client: &reqwest::Client,
    api_url: &str,
    titles: &[String],
) -> Result<PhotoResolution> {
    let mut result = PhotoResolution::default();
    for batch in titles.chunks(50) {
        let body = client
            .get(api_url)
            .query(&[
                ("action", "query"),
                ("format", "json"),
                ("formatversion", "2"),
                ("prop", "imageinfo"),
                ("titles", &batch.join("|")),
                (
                    "iiprop",
                    "url|canonicaltitle|mime|mediatype|size|timestamp|extmetadata",
                ),
                ("iiurlwidth", "1200"),
                (
                    "iiextmetadatafilter",
                    "Artist|Credit|LicenseShortName|LicenseUrl|UsageTerms|NonFree",
                ),
                ("redirects", "1"),
            ])
            .send()
            .await
            .with_context(|| format!("request Commons endpoint {api_url}"))?
            .error_for_status()
            .context("Commons returned an HTTP error")?
            .text()
            .await
            .context("read Commons response")?;
        let batch_result = parse_imageinfo(&body, batch)?;
        result.accepted.extend(batch_result.accepted);
        result.rejected += batch_result.rejected;
    }
    Ok(result)
}

pub fn parse_imageinfo(body: &str, requested_titles: &[String]) -> Result<PhotoResolution> {
    let response: Value =
        serde_json::from_str(body).context("Commons response is not valid JSON")?;
    let query = response
        .get("query")
        .and_then(Value::as_object)
        .context("Commons response is missing query")?;
    let title_map = title_map(query);
    let pages = query
        .get("pages")
        .and_then(Value::as_array)
        .context("Commons response is missing pages")?;
    let pages = pages
        .iter()
        .filter_map(|page| {
            page.get("title")
                .and_then(Value::as_str)
                .map(|title| (title, page))
        })
        .collect::<BTreeMap<_, _>>();

    let mut result = PhotoResolution::default();
    for requested_title in requested_titles {
        let title = resolved_title(requested_title, &title_map);
        let photo = pages
            .get(title.as_str())
            .and_then(|page| page.get("imageinfo"))
            .and_then(Value::as_array)
            .and_then(|imageinfo| imageinfo.first())
            .and_then(photo_from_imageinfo);
        match photo {
            Some(photo) => result.accepted.push(ResolvedPhoto {
                requested_title: requested_title.clone(),
                photo,
            }),
            None => result.rejected += 1,
        }
    }
    Ok(result)
}

fn title_map(query: &serde_json::Map<String, Value>) -> BTreeMap<String, String> {
    ["normalized", "redirects"]
        .into_iter()
        .flat_map(|key| {
            query
                .get(key)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|mapping| {
            Some((
                mapping.get("from")?.as_str()?.to_owned(),
                mapping.get("to")?.as_str()?.to_owned(),
            ))
        })
        .collect()
}

fn resolved_title(requested_title: &str, title_map: &BTreeMap<String, String>) -> String {
    let mut title = requested_title.to_owned();
    let mut visited = BTreeSet::new();
    while visited.insert(title.clone()) {
        let Some(next) = title_map.get(&title) else {
            break;
        };
        title = next.clone();
    }
    title
}

fn photo_from_imageinfo(imageinfo: &Value) -> Option<LicensedPhoto> {
    let metadata = imageinfo.get("extmetadata")?.as_object()?;
    if metadata_value(metadata, "NonFree")
        .is_some_and(|value| metadata_text(value).eq_ignore_ascii_case("true"))
    {
        return None;
    }
    let license = metadata_text(metadata_value(metadata, "LicenseShortName")?);
    if !reusable_license(&license) {
        return None;
    }
    let author = metadata_text(metadata_value(metadata, "Artist")?);
    let attribution = metadata_text(
        metadata_value(metadata, "Credit")
            .unwrap_or_else(|| metadata_value(metadata, "Artist").unwrap()),
    );
    let license_url = metadata_text(metadata_value(metadata, "LicenseUrl")?);
    let url = imageinfo
        .get("thumburl")
        .or_else(|| imageinfo.get("url"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())?
        .to_owned();
    let source_url = imageinfo
        .get("descriptionurl")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())?
        .to_owned();

    (!author.is_empty() && !attribution.is_empty() && !license_url.is_empty()).then_some(
        LicensedPhoto {
            url,
            author,
            license,
            license_url,
            attribution,
            source_url,
        },
    )
}

fn metadata_value<'a>(
    metadata: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Option<&'a str> {
    metadata.get(field)?.get("value")?.as_str()
}

fn reusable_license(license: &str) -> bool {
    let tokens = license
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_uppercase)
        .collect::<Vec<_>>();
    if tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "NC" | "ND" | "NONCOMMERCIAL" | "NODERIVATIVES"
        )
    }) || contains_tokens(&tokens, &["NON", "COMMERCIAL"])
        || contains_tokens(&tokens, &["NO", "DERIVATIVES"])
    {
        return false;
    }
    match tokens.as_slice() {
        [public, domain] if public == "PUBLIC" && domain == "DOMAIN" => true,
        [public, domain, mark, version @ ..]
            if public == "PUBLIC"
                && domain == "DOMAIN"
                && mark == "MARK"
                && version_tokens(version) =>
        {
            true
        }
        [pd] if pd == "PD" => true,
        [cc0, version @ ..] if cc0 == "CC0" && version_tokens(version) => true,
        [cc, zero, version @ ..] if cc == "CC" && zero == "0" && version_tokens(version) => true,
        [cc, by, sa, version @ ..]
            if cc == "CC" && by == "BY" && sa == "SA" && version_tokens(version) =>
        {
            true
        }
        [cc, by, version @ ..] if cc == "CC" && by == "BY" && version_tokens(version) => true,
        _ => false,
    }
}

fn contains_tokens(tokens: &[String], phrase: &[&str]) -> bool {
    tokens
        .windows(phrase.len())
        .any(|tokens| tokens.iter().map(String::as_str).eq(phrase.iter().copied()))
}

fn version_tokens(tokens: &[String]) -> bool {
    tokens
        .iter()
        .all(|token| token.chars().all(|character| character.is_ascii_digit()))
}

fn metadata_text(value: &str) -> String {
    let mut in_tag = false;
    value
        .chars()
        .filter(|character| match character {
            '<' => {
                in_tag = true;
                false
            }
            '>' => {
                in_tag = false;
                false
            }
            _ => !in_tag,
        })
        .collect::<String>()
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        sync::{Arc, Mutex},
    };

    use axum::{
        Json, Router,
        extract::{Query, State},
        routing::get,
    };
    use serde_json::{Value, json};
    use tokio::{net::TcpListener, task::JoinHandle};

    use super::{parse_imageinfo, resolve_commons};

    #[test]
    fn accepts_only_reusable_fixture_photos_and_strips_attribution_html() {
        let requested = vec![
            "File:CC-BY-SA.jpg".to_owned(),
            "File:Public-domain.jpg".to_owned(),
            "File:NC.jpg".to_owned(),
            "File:Missing-license.jpg".to_owned(),
            "File:Non-free.jpg".to_owned(),
        ];

        let result = parse_imageinfo(
            include_str!("../tests/fixtures/commons-imageinfo.json"),
            &requested,
        )
        .unwrap();

        assert_eq!(result.accepted.len(), 2);
        assert_eq!(result.rejected, 3);
        assert_eq!(result.accepted[0].photo.license, "CC BY-SA 4.0");
        assert!(!result.accepted[0].photo.attribution.contains('<'));
        assert!(
            result.accepted[0]
                .photo
                .source_url
                .starts_with("https://commons.wikimedia.org/")
        );
    }

    fn image_page(title: &str, license: &str, non_free: Option<bool>) -> Value {
        let mut metadata = json!({
            "Artist": { "value": "Example author" },
            "Credit": { "value": "Example credit" },
            "LicenseShortName": { "value": license },
            "LicenseUrl": { "value": "https://creativecommons.org/licenses/by/4.0/" }
        });
        if let Some(non_free) = non_free {
            metadata["NonFree"] = json!({ "value": non_free.to_string() });
        }
        json!({
            "title": title,
            "imageinfo": [{
                "url": "https://upload.wikimedia.org/example/image.jpg",
                "descriptionurl": "https://commons.wikimedia.org/wiki/File:Example.jpg",
                "extmetadata": metadata
            }]
        })
    }

    #[test]
    fn accepts_only_known_license_forms_without_nonfree_metadata() {
        let requested = [
            "File:Public-domain.jpg",
            "File:CC0.jpg",
            "File:CC-BY.jpg",
            "File:CC-BY-SA.jpg",
            "File:Spaced-NC.jpg",
            "File:Spaced-ND.jpg",
            "File:Unknown.jpg",
        ]
        .map(str::to_owned);
        let response = json!({
            "query": {
                "pages": [
                    image_page("File:Public-domain.jpg", "pUbLiC DoMaIn", None),
                    image_page("File:CC0.jpg", "Cc-0 1.0", None),
                    image_page("File:CC-BY.jpg", "cc by 4.0", None),
                    image_page("File:CC-BY-SA.jpg", "CC BY SA 4.0", None),
                    image_page("File:Spaced-NC.jpg", "CC BY NC 4.0", None),
                    image_page("File:Spaced-ND.jpg", "CC BY ND 4.0", None),
                    image_page("File:Unknown.jpg", "Not public domain", None)
                ]
            }
        });

        let result = parse_imageinfo(&response.to_string(), &requested).unwrap();

        assert_eq!(result.rejected, 3);
        assert_eq!(
            result
                .accepted
                .iter()
                .map(|photo| photo.requested_title.as_str())
                .collect::<Vec<_>>(),
            [
                "File:Public-domain.jpg",
                "File:CC0.jpg",
                "File:CC-BY.jpg",
                "File:CC-BY-SA.jpg",
            ]
        );
    }

    #[test]
    fn preserves_the_original_title_through_normalized_redirects() {
        let requested = vec!["File:Requested title.jpg".to_owned()];
        let response = json!({
            "query": {
                "normalized": [{ "from": "File:Requested title.jpg", "to": "File:Normalized title.jpg" }],
                "redirects": [{ "from": "File:Normalized title.jpg", "to": "File:Canonical title.jpg" }],
                "pages": [image_page("File:Canonical title.jpg", "CC BY 4.0", Some(false))]
            }
        });

        let result = parse_imageinfo(&response.to_string(), &requested).unwrap();

        assert_eq!(result.rejected, 0);
        assert_eq!(
            result.accepted[0].requested_title,
            "File:Requested title.jpg"
        );
    }

    #[derive(Clone)]
    struct CommonsFixture(Arc<Mutex<Vec<BTreeMap<String, String>>>>);

    async fn commons_fixture(
        State(requests): State<CommonsFixture>,
        Query(query): Query<BTreeMap<String, String>>,
    ) -> Json<Value> {
        requests.0.lock().unwrap().push(query.clone());
        let pages = query["titles"]
            .split('|')
            .map(|title| image_page(title, "CC BY 4.0", Some(false)))
            .collect::<Vec<_>>();
        Json(json!({ "query": { "pages": pages } }))
    }

    async fn fixture_server(requests: CommonsFixture) -> (String, JoinHandle<()>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new()
                    .route("/commons", get(commons_fixture))
                    .with_state(requests),
            )
            .await
            .unwrap();
        });
        (format!("http://{address}/commons"), server)
    }

    #[tokio::test]
    async fn fetches_batches_of_fifty_with_encoded_titles_and_preserved_duplicates() {
        let requests = CommonsFixture(Arc::new(Mutex::new(Vec::new())));
        let (api_url, server) = fixture_server(requests.clone()).await;
        let mut titles = (0..51)
            .map(|index| format!("File:Image {index}.jpg"))
            .collect::<Vec<_>>();
        titles.push("File:Image 0.jpg".to_owned());
        titles.push("File:Names & spaces.jpg".to_owned());

        let result = resolve_commons(&reqwest::Client::new(), &api_url, &titles)
            .await
            .unwrap();
        server.abort();

        assert_eq!(result.rejected, 0);
        assert_eq!(
            result
                .accepted
                .iter()
                .map(|photo| photo.requested_title.as_str())
                .collect::<Vec<_>>(),
            titles.iter().map(String::as_str).collect::<Vec<_>>(),
        );
        let requests = requests.0.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0]["titles"].split('|').count(), 50);
        assert_eq!(requests[1]["titles"].split('|').count(), 3);
        assert_eq!(
            requests
                .iter()
                .flat_map(|query| query["titles"].split('|'))
                .collect::<Vec<_>>(),
            titles.iter().map(String::as_str).collect::<Vec<_>>(),
        );
        assert_eq!(
            requests[0]["iiprop"],
            "url|canonicaltitle|mime|mediatype|size|timestamp|extmetadata"
        );
        assert_eq!(requests[0]["iiurlwidth"], "1200");
        assert_eq!(
            requests[0]["iiextmetadatafilter"],
            "Artist|Credit|LicenseShortName|LicenseUrl|UsageTerms|NonFree"
        );
    }
}
