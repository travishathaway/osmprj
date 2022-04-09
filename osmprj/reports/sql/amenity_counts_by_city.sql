WITH count_data as (
	SELECT
		pl.name as city
		, pl.geom
		, pp.osm_type as amenity
		, pl.area / 1000000.0 as area_sq_km
		, count(*) as count
	FROM
		amenity_point pp
	JOIN
		place_polygon pl
	ON
		ST_Contains(pl.geom, pp.geom)
	WHERE
	(
		pp.osm_type = %(amenity)s
    )
	AND
		pl.name in %(cities)s
	AND
		round(ST_Area(ST_Transform(pl.geom, 4326)::geography) / 1000) / 1000 > 1
	GROUP BY
		pp.osm_type, pl.name, pl.geom, pl.area
)

SELECT
	city as {city}
	, amenity as {amenity}
	, area_sq_km as {area_sq_km}
	, count as {count}
	, count / area_sq_km as {amenity_per_sq_km}
FROM
	count_data
ORDER BY
	5 asc;
