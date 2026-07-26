## MODIFIED Requirements

### Requirement: Custom room name

The room creation modal SHALL provide a text input field for the room name, instead of a hardcoded button.

#### Scenario: Room name text input
- **WHEN** the create room modal opens
- **THEN** a text input field SHALL be displayed with the room name label
- **AND** the input field SHALL be auto-focused
- **AND** the user SHALL be able to type a custom room name

#### Scenario: Custom name used on create
- **WHEN** the user clicks "创建房间" in the modal
- **THEN** the entered room name SHALL be used to create the room
- **AND** if the name is empty, a default name SHALL be generated
