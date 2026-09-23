CREATE TABLE source_playgrounds (
    source text NOT NULL CHECK (source IN ('openstreetmap', 'sofiaplan')),
    external_id text NOT NULL,
    raw_data jsonb NOT NULL,
    normalized_name text,
    location geography(Point, 4326) NOT NULL,
    source_date timestamptz,
    date_meaning text CHECK (date_meaning IN ('observation', 'source_update')),
    imported_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (source, external_id)
);

CREATE INDEX source_playgrounds_location_gist_idx
    ON source_playgrounds USING gist (location);

ALTER TABLE playgrounds
    ADD COLUMN address text,
    ADD COLUMN surface text,
    ADD COLUMN fenced boolean,
    ADD COLUMN ownership text,
    ADD COLUMN access text,
    ADD COLUMN fee text,
    ADD COLUMN municipal_status text,
    ADD COLUMN ordinance_compliant boolean,
    ADD COLUMN repairs text,
    ADD COLUMN notes text,
    ADD COLUMN primary_source text NOT NULL DEFAULT 'openstreetmap'
        CHECK (primary_source IN ('openstreetmap', 'sofiaplan')),
    ADD COLUMN photos jsonb NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN source_values jsonb NOT NULL DEFAULT '[]'::jsonb,
    ADD CONSTRAINT playgrounds_photos_array_check
        CHECK (jsonb_typeof(photos) = 'array'),
    ADD CONSTRAINT playgrounds_source_values_array_check
        CHECK (jsonb_typeof(source_values) = 'array');

CREATE TABLE playground_source_links (
    playground_id text NOT NULL REFERENCES playgrounds(id) ON DELETE CASCADE,
    source text NOT NULL,
    external_id text NOT NULL,
    match_method text NOT NULL CHECK (match_method IN ('unmatched', 'proximity')),
    match_distance_meters double precision
        CHECK (match_distance_meters IS NULL OR match_distance_meters >= 0),
    PRIMARY KEY (source, external_id),
    FOREIGN KEY (source, external_id)
        REFERENCES source_playgrounds(source, external_id) ON DELETE CASCADE
);

CREATE INDEX playground_source_links_playground_idx
    ON playground_source_links(playground_id);
