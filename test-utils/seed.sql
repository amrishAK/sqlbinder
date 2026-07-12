BEGIN;

CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    age INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS teams (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    owner_user_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS team_members (
    team_id BIGINT NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'member',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (team_id, user_id)
);

CREATE TABLE IF NOT EXISTS datasets (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    owner_team_id BIGINT REFERENCES teams(id) ON DELETE SET NULL,
    visibility TEXT NOT NULL DEFAULT 'private',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS dataset_items (
    id BIGSERIAL PRIMARY KEY,
    dataset_id BIGINT NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    external_id TEXT NOT NULL,
    image_url TEXT,
    label TEXT,
    tag TEXT,
    source TEXT,
    provenance TEXT,
    width INTEGER,
    height INTEGER,
    hash TEXT,
    split_assignment TEXT,
    licensing TEXT,
    score NUMERIC(10,4),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (dataset_id, external_id)
);

CREATE TABLE IF NOT EXISTS item_tags (
    item_id BIGINT NOT NULL REFERENCES dataset_items(id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    PRIMARY KEY (item_id, tag)
);

CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at);
CREATE INDEX IF NOT EXISTS idx_teams_owner_user_id ON teams(owner_user_id);
CREATE INDEX IF NOT EXISTS idx_team_members_user_id ON team_members(user_id);
CREATE INDEX IF NOT EXISTS idx_dataset_items_dataset_id ON dataset_items(dataset_id);
CREATE INDEX IF NOT EXISTS idx_dataset_items_label ON dataset_items(label);
CREATE INDEX IF NOT EXISTS idx_dataset_items_tag ON dataset_items(tag);
CREATE INDEX IF NOT EXISTS idx_dataset_items_hash ON dataset_items(hash);
CREATE INDEX IF NOT EXISTS idx_item_tags_tag ON item_tags(tag);

INSERT INTO users (name, email, age, status) VALUES
('Alice', 'alice@example.com', 29, 'active'),
('Bob', 'bob@example.com', 34, 'active'),
('Charlie', 'charlie@example.com', 41, 'disabled'),
('Diana', 'diana@example.com', 26, 'active'),
('Eve', 'eve@example.com', 38, 'active')
ON CONFLICT (email) DO NOTHING;

INSERT INTO teams (name, owner_user_id) VALUES
('platform', 1),
('ml', 2),
('infra', 4)
ON CONFLICT (name) DO NOTHING;

INSERT INTO team_members (team_id, user_id, role) VALUES
(1, 1, 'owner'),
(1, 2, 'member'),
(1, 5, 'member'),
(2, 2, 'owner'),
(2, 3, 'member'),
(3, 4, 'owner'),
(3, 1, 'advisor')
ON CONFLICT DO NOTHING;

INSERT INTO datasets (name, description, owner_team_id, visibility) VALUES
('sample-images', 'Image metadata catalog for basic and complex queries', 2, 'public'),
('documents', 'Document metadata and classification catalog', 1, 'private'),
('events', 'Event-like records for aggregation and filtering tests', 3, 'internal'),
('audit-log', 'Write-heavy dataset for mutation and lookup tests', 3, 'internal')
ON CONFLICT (name) DO NOTHING;

INSERT INTO dataset_items
(dataset_id, external_id, image_url, label, tag, source, provenance, width, height, hash, split_assignment, licensing, score, metadata)
SELECT
    d.id,
    v.external_id,
    v.image_url,
    v.label,
    v.tag,
    v.source,
    v.provenance,
    v.width,
    v.height,
    v.hash,
    v.split_assignment,
    v.licensing,
    v.score,
    v.metadata::jsonb
FROM datasets d
JOIN (
    VALUES
        ('sample-images', 'img-001', 'https://example.com/img-001.jpg', 'cat', 'animal', 'camera', 'upload', 640, 480, 'hash001', 'train', 'cc-by', 0.9812, '{"format":"jpg","colors":["gray","white"]}'),
        ('sample-images', 'img-002', 'https://example.com/img-002.jpg', 'dog', 'animal', 'camera', 'upload', 800, 600, 'hash002', 'val', 'cc-by', 0.8765, '{"format":"jpg","colors":["brown"]}'),
        ('sample-images', 'img-003', 'https://example.com/img-003.jpg', 'car', 'vehicle', 'api', 'generated', 1024, 768, 'hash003', 'test', 'internal', 0.7444, '{"format":"png","weather":"night"}'),
        ('sample-images', 'img-004', NULL, 'bird', 'animal', 'import', 'manual', NULL, NULL, 'hash004', 'train', 'cc-by', 0.6655, '{"notes":"missing url for null-path testing"}'),
        ('documents', 'doc-001', NULL, 'invoice', 'finance', 'scanner', 'import', NULL, NULL, 'hash005', 'train', 'internal', 0.9921, '{"pages":2,"language":"en"}'),
        ('documents', 'doc-002', NULL, 'receipt', 'finance', 'scanner', 'import', NULL, NULL, 'hash006', 'val', 'internal', 0.8322, '{"pages":1,"language":"en"}'),
        ('events', 'evt-001', NULL, 'signup', 'user', 'api', 'stream', NULL, NULL, 'hash007', 'prod', 'internal', 0.5000, '{"region":"eu","channel":"web"}'),
        ('events', 'evt-002', NULL, 'purchase', 'commerce', 'api', 'stream', NULL, NULL, 'hash008', 'prod', 'internal', 0.7100, '{"region":"eu","channel":"mobile"}'),
        ('audit-log', 'log-001', NULL, 'update', 'system', 'db', 'trigger', NULL, NULL, 'hash009', 'prod', 'internal', 0.3000, '{"table":"users","op":"update"}'),
        ('audit-log', 'log-002', NULL, 'insert', 'system', 'db', 'trigger', NULL, NULL, 'hash010', 'prod', 'internal', 0.2900, '{"table":"datasets","op":"insert"}')
) AS v(dataset_name, external_id, image_url, label, tag, source, provenance, width, height, hash, split_assignment, licensing, score, metadata)
ON d.name = v.dataset_name
ON CONFLICT (dataset_id, external_id) DO NOTHING;

INSERT INTO item_tags (item_id, tag)
SELECT id, tag FROM dataset_items WHERE tag IS NOT NULL
ON CONFLICT DO NOTHING;

INSERT INTO item_tags (item_id, tag)
SELECT id, 'featured' FROM dataset_items WHERE id IN (1, 3, 5, 7)
ON CONFLICT DO NOTHING;

COMMIT;