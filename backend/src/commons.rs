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
    let response: Value = serde_json::from_str(body).context("Commons response is not valid JSON")?;
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
        .flat_map(|key| query.get(key).and_then(Value::as_array).into_iter().flatten())
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
    if metadata_text(metadata_value(metadata, "NonFree")?).eq_ignore_ascii_case("true") {
        return None;
    }
    let license = metadata_text(metadata_value(metadata, "LicenseShortName")?);
    if !reusable_license(&license) {
        return None;
    }
    let author = metadata_text(metadata_value(metadata, "Artist")?);
    let attribution = metadata_text(
        metadata_value(metadata, "Credit").unwrap_or_else(|| metadata_value(metadata, "Artist").unwrap()),
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

    (!author.is_empty() && !attribution.is_empty() && !license_url.is_empty()).then_some(LicensedPhoto {
        url,
        author,
        license,
        license_url,
        attribution,
        source_url,
    })
}

fn metadata_value<'a>(metadata: &'a serde_json::Map<String, Value>, field: &str) -> Option<&'a str> {
    metadata.get(field)?.get("value")?.as_str()
}

fn reusable_license(license: &str) -> bool {
    let license = license.trim().to_ascii_lowercase();
    if license.contains("noncommercial")
        || license.contains("non-commercial")
        || license.contains("-nc")
        || license.contains("noderivatives")
        || license.contains("no-derivatives")
        || license.contains("-nd")
    {
        return false;
    }
    license.contains("public domain")
        || license.starts_with("cc0")
        || license.starts_with("cc by-sa")
        || license.starts_with("cc by ")
}

fn metadata_text(value: &str) -> String {
    let mut in_tag = false;
    value
        .chars()
        .filter(|character| match character {
            '<' => { in_tag = true; false }
            '>' => { in_tag = false; false }
            _ => !in_tag,
        })
        .collect::<String>()
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::parse_imageinfo;

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
        assert!(result.accepted[0]
            .photo
            .source_url
            .starts_with("https://commons.wikimedia.org/"));
    }
}
