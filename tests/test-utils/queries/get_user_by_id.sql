SELECT
    id AS id,
    name AS name,
    email AS email,
    age AS age,
    status AS status
FROM users
WHERE id = $user_id;
