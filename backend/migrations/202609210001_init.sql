CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE neighborhoods (
    id text PRIMARY KEY,
    name text NOT NULL,
    boundary geometry(MultiPolygon, 4326) NOT NULL
);

CREATE INDEX neighborhoods_boundary_gist_idx ON neighborhoods USING gist (boundary);

CREATE TABLE playgrounds (
    id text PRIMARY KEY,
    name text,
    location geography(Point, 4326) NOT NULL,
    capabilities text[] NOT NULL DEFAULT '{}',
    min_age smallint,
    max_age smallint,
    photo_urls text[] NOT NULL DEFAULT '{}',
    source_url text NOT NULL,
    source_updated_at timestamptz,
    CONSTRAINT playgrounds_age_bounds_check CHECK (
        (min_age IS NULL OR min_age BETWEEN 0 AND 18)
        AND (max_age IS NULL OR max_age BETWEEN 0 AND 18)
        AND (min_age IS NULL OR max_age IS NULL OR min_age <= max_age)
    )
);

CREATE INDEX playgrounds_location_gist_idx ON playgrounds USING gist (location);
CREATE INDEX playgrounds_capabilities_gin_idx ON playgrounds USING gin (capabilities);

CREATE TABLE playground_neighborhoods (
    playground_id text NOT NULL REFERENCES playgrounds(id) ON DELETE CASCADE,
    neighborhood_id text NOT NULL REFERENCES neighborhoods(id) ON DELETE CASCADE,
    PRIMARY KEY (playground_id, neighborhood_id)
);

CREATE INDEX playground_neighborhoods_neighborhood_idx
    ON playground_neighborhoods (neighborhood_id, playground_id);

