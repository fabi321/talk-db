INSERT INTO sendungen
(sid, name, url, datum) VALUES (?,?,?,?)
RETURNING seid as "seid: i64";
