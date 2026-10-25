# diagramdiff

diagramdiff shows what changed between two versions of a diagram file in the diagram's own terms: Nodes and Edges added and removed, not lines of JSON.

## Diagrams

**Format**:
A kind of diagram file diagramdiff reads, such as Excalidraw, JSON Canvas, draw.io or Mermaid. Each Format decides what a Key is.
_Avoid_: file type, parser

**Diagram**:
The Nodes and Edges read from one diagram file.
_Avoid_: drawing, scene

**Node**:
Any part of a Diagram that is not an Edge: a box, a standalone text, a frame, a line or a freedraw stroke.
_Avoid_: element, shape

**Edge**:
A connection drawn between Nodes; either end may be unconnected. In Excalidraw only an arrow is an Edge.
_Avoid_: link, connector

**From / To**:
The Nodes an Edge starts and ends at. An end that is not bound to a Node in the same Diagram is unconnected.
_Avoid_: source/target, start/end

**Key**:
The identity of a Node or Edge, as its Format defines it, such as an Excalidraw element id. A Node or Edge in the old Diagram is the same as one in the new Diagram only when their Keys are equal.
_Avoid_: identity, fingerprint

**Kind**:
What sort of Node or Edge it is, named as its Format names it: `rectangle`, `arrow`, `frame`.
_Avoid_: type

**Label**:
The text a reader sees on a Node or Edge. A frame's Label is its name.
_Avoid_: text, title

**Tombstone**:
An element that a Format keeps in the file but marks as deleted. It is not part of the Diagram.
_Avoid_: deleted element

## Changes

**Change**:
A difference between the old and the new Diagram that diagramdiff reports.
_Avoid_: diff, delta

**Added**:
A Node or Edge whose Key is only in the new Diagram.

**Removed**:
A Node or Edge whose Key is only in the old Diagram.

**Layout**:
Where and how big a Node or Edge is drawn: position, size, rotation and route.
_Avoid_: geometry, placement

**Noise**:
A difference between two files that is not a Change, such as Layout, style or the order of elements in the file.
_Avoid_: cosmetic change

**Membership**:
The groups and the frame a Node belongs to. Moving into or out of one is a Change, even where the Format expresses it only through Layout.
_Avoid_: parent, containment
