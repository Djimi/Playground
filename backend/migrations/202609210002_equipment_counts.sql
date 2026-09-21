ALTER TABLE playgrounds
    ADD COLUMN equipment_counts jsonb NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE playgrounds
    ADD CONSTRAINT playgrounds_equipment_counts_object_check
    CHECK (jsonb_typeof(equipment_counts) = 'object');
