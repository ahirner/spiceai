SELECT * FROM hits WHERE "URL" LIKE '%google%' ORDER BY to_timestamp("EventTime") LIMIT 10;
