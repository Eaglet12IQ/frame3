-- Basic schema

CREATE TABLE IF NOT EXISTS iss_fetch_log (
    id BIGSERIAL PRIMARY KEY,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    source_url TEXT NOT NULL,
    payload JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS telemetry_legacy (
    id BIGSERIAL PRIMARY KEY,
    recorded_at TIMESTAMPTZ NOT NULL,
    voltage NUMERIC(6,2) NOT NULL,
    temp NUMERIC(6,2) NOT NULL,
    source_file TEXT NOT NULL,
    is_valid BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS cms_pages (
    id BIGSERIAL PRIMARY KEY,
    slug TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cms_blocks (
    id BIGSERIAL PRIMARY KEY,
    slug TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed with deliberately unsafe content for XSS practice
INSERT INTO cms_pages(slug, title, body)
VALUES
('welcome', 'Добро пожаловать', '<h3>Демо контент</h3><p>Этот текст хранится в БД</p>'),
('unsafe', 'Небезопасный пример', '<script>console.log("XSS training")
</script><p>Если вы видите всплывашку значит защита не работает</p>')
ON CONFLICT DO NOTHING;

INSERT INTO cms_blocks(slug, title, content, is_active)
VALUES
('dashboard_experiment', 'Космические факты', '<div class="alert alert-info"><h5>Интересные факты о космосе</h5><ul><li>МКС движется со скоростью около 28 000 км/ч</li><li>JWST видит свет, излученный 13.5 млрд лет назад</li><li>Астрономические события можно наблюдать с Земли</li></ul><p><small>Обновлено: ' || NOW() || '</small></p></div>', TRUE)
ON CONFLICT DO NOTHING;

-- OSDR
CREATE TABLE IF NOT EXISTS osdr_items(
    id BIGSERIAL PRIMARY KEY,
    dataset_id TEXT,
    title TEXT,
    status TEXT,
    updated_at TIMESTAMPTZ,
    inserted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    raw JSONB NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_osdr_dataset_id
 ON osdr_items(dataset_id) WHERE dataset_id IS NOT NULL;

-- универсальный кэш космоданных
CREATE TABLE IF NOT EXISTS space_cache(
    id BIGSERIAL PRIMARY KEY,
    source TEXT NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    payload JSONB NOT NULL
);
CREATE INDEX IF NOT EXISTS ix_space_cache_source ON space_cache(source,fetched_at DESC);
