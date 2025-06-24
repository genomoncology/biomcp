import pytest


def _recursive_extract(current_value, key_path, path_index):
    """Recursively extract values based on the key path."""
    if path_index >= len(key_path):
        if isinstance(current_value, list):
            yield from current_value
        else:
            yield current_value

    else:
        k = key_path[path_index]
        if isinstance(current_value, dict):
            next_value = current_value.get(k)
            if next_value is not None:
                yield from _recursive_extract(
                    next_value,
                    key_path,
                    path_index + 1,
                )

        elif isinstance(current_value, list):
            for item in current_value:
                if isinstance(item, dict):
                    next_value = item.get(k)
                    if next_value is not None:
                        yield from _recursive_extract(
                            next_value,
                            key_path,
                            path_index + 1,
                        )


def iter_value(field_map: dict, data: dict | list, key: str):
    """Iterates through a nested structure, yielding all values encountered."""
    if isinstance(data, dict):
        # Handle new format with cBioPortal summary
        hits = data["variants"] if "variants" in data else data.get("hits", [])
    else:
        hits = data
    key_path = field_map.get(key, [key])

    # num = variant number for tracking each individual variant
    for num, hit in enumerate(hits, 1):
        for value in _recursive_extract(hit, key_path, 0):
            yield num, value


@pytest.fixture(scope="module")
def it() -> callable:
    return iter_value
