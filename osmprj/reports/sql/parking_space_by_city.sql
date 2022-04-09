SELECT
	pl.name AS {city}
	--, SUM(ST_Area(ap.geom) / 1000000) as {parking_area_sq_km}
	--, SUM(ST_Area(pl.geom) / 1000000) as {city_area_sq_km}
	, SUM(ap.area) / 1000000.0 as {parking_area_sq_km}
	, pl.area / 1000000.0 as {city_area_sq_km}
	, (SUM(ap.area) / pl.area) as {percentage_parking_area}
FROM
	amenity_polygon ap
JOIN
	place_polygon pl
ON
	ST_Within(ap.geom, pl.geom)
WHERE
	ap.osm_type = 'parking'
AND
	pl.name IN %(cities)s
GROUP BY
	pl.name, pl.area
ORDER BY
    4 DESC
