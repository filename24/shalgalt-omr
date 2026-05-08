-- P1 — sidecar column for the editor backdrop image.
--
-- Rule 3 keeps `templates.json_schema` as the pure `OmrTemplate` serialization
-- (markers + groups + answer key). The backdrop is a machine-local visual aid
-- that lives in the user's app-data dir and would not survive sharing the
-- template JSON, so it is stored here as a separate column. NULL means "no
-- backdrop yet" (e.g. an in-flight draft, or a template imported from another
-- machine).

ALTER TABLE templates ADD COLUMN backdrop_path TEXT;
