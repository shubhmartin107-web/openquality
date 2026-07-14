import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

from openquality.alerting.core import IncidentManager, StdoutAlertChannel


class FakeMonitorResult:
    def __init__(self):
        self.monitor_id = "test-mon"
        self.monitor_type = "volume"
        self.table_name = "test_table"
        self.alert = True
        self.severity = "CRITICAL"
        self.score = 100.0
        self.threshold = 50.0
        self.message = "Test alert"
        self.details = {"row_count": 0}
        self.timestamp = None


def test_incident_manager():
    mgr = IncidentManager()
    channel = StdoutAlertChannel()
    mgr.add_channel(channel)
    result = FakeMonitorResult()
    incident = mgr.register(result, ["Possible pipeline failure"])
    assert incident.monitor_id == "test-mon"
    assert incident.severity == "CRITICAL"
    assert not incident.resolved


def test_resolve_incident():
    mgr = IncidentManager()
    result = FakeMonitorResult()
    incident = mgr.register(result, [])
    resolved = mgr.resolve(incident.id)
    assert resolved is not None
    assert resolved.resolved


def test_list_incidents():
    mgr = IncidentManager()
    assert len(mgr.list()) == 0
    result = FakeMonitorResult()
    mgr.register(result, [])
    assert len(mgr.list()) == 1
