CREATE TYPE item_slot AS ENUM ('primary', 'secondary', 'melee');
CREATE TYPE merc AS ENUM ('Scout', 'Soldier', 'Pyro', 'Demoman', 'Heavy', 'Engineer', 'Medic', 'Sniper', 'Spy');

CREATE TABLE IF NOT EXISTS weapons (
    id INT PRIMARY KEY,
    name TEXT NOT NULL,
    stock BOOLEAN NOT NULL,
    item_name TEXT NOT NULL,
    item_slot item_slot NOT NULL,
    image_url TEXT NOT NULL,
    image_url_large TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS weapon_used_by_classes (
    weapon_id INT NOT NULL REFERENCES weapons(id),
    merc merc NOT NULL,
    PRIMARY KEY (weapon_id, merc)
);

CREATE TABLE IF NOT EXISTS weapon_per_class_loadout_slots (
    weapon_id INT NOT NULL REFERENCES weapons(id),
    merc merc NOT NULL,
    loadout_slot item_slot NOT NULL,
    PRIMARY KEY (weapon_id, merc)
);

CREATE VIEW IF NOT EXISTS weapon_details AS
SELECT
    w.id,
    w.name,
    w.stock,
    w.item_name,
    COALESCE(pcls.loadout_slot, w.item_slot) AS item_slot,
    w.image_url,
    w.image_url_large,
    ubc.merc AS merc  -- can be NULL
FROM weapons AS w
LEFT JOIN weapon_used_by_classes AS ubc 
ON w.id = ubc.weapon_id
LEFT JOIN weapon_per_class_loadout_slots AS pcls
ON w.id = pcls.weapon_id
AND (pcls.merc IS NULL OR ubc.merc IS NULL OR ubc.merc = pcls.merc)
ORDER BY w.stock DESC, w.id;

CREATE TABLE IF NOT EXISTS loadouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL,
    merc merc NOT NULL,
    "primary" INT NOT NULL REFERENCES weapons(id),
    secondary INT NOT NULL REFERENCES weapons(id),
    melee INT NOT NULL REFERENCES weapons(id),
    name TEXT NOT NULL,
    playstyle TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE VIEW full_loadouts AS
SELECT
    l.*,
    jsonb_build_object(
        'id', wd_primary.id,
        'name', wd_primary.name,
        'stock', wd_primary.stock,
        'item_name', wd_primary.item_name,
        'item_slot', wd_primary.item_slot,
        'image_url', wd_primary.image_url,
        'image_url_large', wd_primary.image_url_large
    ) AS primary_weapon,
    jsonb_build_object(
        'id', wd_secondary.id,
        'name', wd_secondary.name,
        'stock', wd_secondary.stock,
        'item_name', wd_secondary.item_name,
        'item_slot', wd_secondary.item_slot,
        'image_url', wd_secondary.image_url,
        'image_url_large', wd_secondary.image_url_large
    ) AS secondary_weapon,
    jsonb_build_object(
        'id', wd_melee.id,
        'name', wd_melee.name,
        'stock', wd_melee.stock,
        'item_name', wd_melee.item_name,
        'item_slot', wd_melee.item_slot,
        'image_url', wd_melee.image_url,
        'image_url_large', wd_melee.image_url_large
    ) AS melee_weapon
FROM loadouts l
LEFT JOIN weapons wd_primary ON l.primary = wd_primary.id
LEFT JOIN weapons wd_secondary ON l.secondary = wd_secondary.id
LEFT JOIN weapons wd_melee ON l.melee = wd_melee.id;


CREATE OR REPLACE FUNCTION check_loadout_weapons() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.primary IS NOT NULL AND NOT EXISTS (
        SELECT 1
        FROM weapon_details
        WHERE (merc = NEW.merc OR merc IS NULL)
        AND item_slot = 'primary'
        AND id = NEW.primary
    ) THEN
        RAISE EXCEPTION 'Invalid primary weapon for the provided merc'
        USING ERRCODE = 'TF001';
    END IF;

    IF NEW.secondary IS NOT NULL AND NOT EXISTS (
        SELECT 1
        FROM weapon_details
        WHERE (merc = NEW.merc OR merc IS NULL)
        AND item_slot = 'secondary'
        AND id = NEW.secondary
    ) THEN
        RAISE EXCEPTION 'Invalid secondary weapon for the provided merc'
        USING ERRCODE = 'TF002';
    END IF;

    IF NEW.melee IS NOT NULL AND NOT EXISTS (
        SELECT 1
        FROM weapon_details
        WHERE (merc = NEW.merc OR merc IS NULL)
        AND item_slot = 'melee'
        AND id = NEW.melee
    ) THEN
        RAISE EXCEPTION 'Invalid melee weapon for the provided merc'
        USING ERRCODE = 'TF003';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER validate_loadout_weapons_insert
BEFORE INSERT ON loadouts
FOR EACH ROW
EXECUTE FUNCTION check_loadout_weapons();

CREATE TRIGGER validate_loadout_weapons_update
BEFORE UPDATE ON loadouts
FOR EACH ROW
EXECUTE FUNCTION check_loadout_weapons();

CREATE OR REPLACE FUNCTION update_updated_at_column() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_updated_at
BEFORE UPDATE ON loadouts
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();