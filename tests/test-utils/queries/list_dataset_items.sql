SELECT
    item.external_id AS external_id,
    item.label AS label,
    item.tag AS tag,
    item.score AS score
FROM dataset_items AS item
JOIN datasets AS dataset ON dataset.id = item.dataset_id
WHERE dataset.name = $dataset_name
ORDER BY item.external_id;
