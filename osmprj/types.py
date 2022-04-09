from typing import TypedDict, NewType


class OSMCityData(TypedDict):
    name: str
    bundesland: str
    population: int
    osm_data_link: str
    bbox: list[float]
    output: str
    output_type: str
