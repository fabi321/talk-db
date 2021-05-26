BEGIN;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS gaeste (
	gid INTEGER PRIMARY KEY,
	name TEXT UNIQUE NOT NULL,
	title TEXT,
	party TEXT,
	biografie TEXT,
	url TEXT
);

CREATE TABLE IF NOT EXISTS shows (
	sid INTEGER PRIMARY KEY,
	name TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS sendungen (
	seid INTEGER PRIMARY KEY,
	sid REFERENCES shows (sid) ON DELETE CASCADE ON UPDATE NO ACTION,
	name TEXT NOT NULL DEFAULT '',
	url TEXT NOT NULL,
	datum TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS gastsendung (
	seid REFERENCES sendungen (seid) ON DELETE CASCADE ON UPDATE NO ACTION,
	gid REFERENCES gaeste (gid) ON DELETE CASCADE ON UPDATE NO ACTION,
	PRIMARY KEY (seid, gid)
);

CREATE UNIQUE INDEX IF NOT EXISTS surl ON sendungen (url);
CREATE INDEX IF NOT EXISTS sname ON sendungen (name);
CREATE INDEX IF NOT EXISTS sdatum ON sendungen (datum);
CREATE UNIQUE INDEX IF NOT EXISTS gname ON gaeste (name);
-- No index for shows(name) as shows is expected to be very small (5 entries by now)

INSERT OR IGNORE INTO shows (sid, name) VALUES (0, 'Anne Will');
INSERT OR IGNORE INTO shows (sid, name) VALUES (1, 'Hart aber fair');
INSERT OR IGNORE INTO shows (sid, name) VALUES (2, 'Maischberger - Die Woche');
INSERT OR IGNORE INTO shows (sid, name) VALUES (3, 'Maybrit Illner');
INSERT OR IGNORE INTO shows (sid, name) VALUES (4, 'Markus Lanz');

COMMIT;
