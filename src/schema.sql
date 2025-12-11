CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    target_hours REAL,
    parent INTEGER,
    archived BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (parent)
        REFERENCES projects (id)
);

CREATE TABLE IF NOT EXISTS work (
    time_start INTEGER NOT NULL PRIMARY KEY,
    duration INTEGER NOT NULL,
    project_id INTEGER,
    FOREIGN KEY (project_id)
        REFERENCES projects (id)
);

CREATE VIEW IF NOT EXISTS work_totals AS 
	SELECT
		project_id,
		SUM(duration) AS duration
	FROM
		work
	GROUP BY
		project_id
;