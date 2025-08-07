"""
OpenFDA integration for BioMCP.

Provides access to FDA drug labels, adverse events, device data,
drug approvals, recalls, and shortage information.
"""

from .adverse_events import (
    search_adverse_events,
    get_adverse_event,
)
from .drug_labels import (
    search_drug_labels,
    get_drug_label,
)
from .device_events import (
    search_device_events,
    get_device_event,
)
from .drug_approvals import (
    search_drug_approvals,
    get_drug_approval,
)
from .drug_recalls import (
    search_drug_recalls,
    get_drug_recall,
)
from .drug_shortages import (
    search_drug_shortages,
    get_drug_shortage,
)

__all__ = [
    "get_adverse_event",
    "get_device_event",
    "get_drug_approval",
    "get_drug_label",
    "get_drug_recall",
    "get_drug_shortage",
    "search_adverse_events",
    "search_device_events",
    "search_drug_approvals",
    "search_drug_labels",
    "search_drug_recalls",
    "search_drug_shortages",
]
