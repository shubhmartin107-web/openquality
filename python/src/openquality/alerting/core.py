from __future__ import annotations
import uuid
from datetime import datetime, timezone
from typing import Optional
from dataclasses import dataclass, field


@dataclass
class Incident:
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    monitor_id: str = ""
    severity: str = "WARNING"
    message: str = ""
    detail: dict = field(default_factory=dict)
    root_cause_hints: list[str] = field(default_factory=list)
    timestamp: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    resolved: bool = False
    resolved_at: Optional[datetime] = None
    acked: bool = False


class StdoutAlertChannel:
    def __init__(self, name: str = "stdout"):
        self.name = name

    def send(self, incident: "Incident"):
        ts = incident.timestamp.strftime("%Y-%m-%dT%H:%M:%S.%fZ")
        print(f"[{ts}] [{incident.severity}] {incident.message} — monitor={incident.monitor_id} hints={incident.root_cause_hints}")
        for hint in incident.root_cause_hints:
            print(f"  └─ root cause hint: {hint}")


class IncidentManager:
    def __init__(self):
        self.incidents: list[Incident] = []
        self.channels: list[StdoutAlertChannel] = []

    def add_channel(self, channel: StdoutAlertChannel):
        self.channels.append(channel)

    def register(self, monitor_result, hints: list[str] | None = None) -> Incident:
        incident = Incident(
            monitor_id=monitor_result.monitor_id,
            severity=monitor_result.severity,
            message=monitor_result.message,
            detail={"monitor_result": vars(monitor_result)},
            root_cause_hints=hints or [],
        )
        self.incidents.append(incident)
        for channel in self.channels:
            channel.send(incident)
        return incident

    def resolve(self, incident_id: str) -> Optional[Incident]:
        for inc in self.incidents:
            if inc.id == incident_id:
                inc.resolved = True
                inc.resolved_at = datetime.now(timezone.utc)
                return inc
        return None

    def list(self, resolved: Optional[bool] = None) -> list[Incident]:
        if resolved is None:
            return self.incidents
        return [i for i in self.incidents if i.resolved == resolved]

    def get_by_monitor(self, monitor_id: str) -> list[Incident]:
        return [i for i in self.incidents if i.monitor_id == monitor_id]
