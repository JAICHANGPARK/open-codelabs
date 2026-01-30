# Feature Specifications

Detailed descriptions of all Open Codelabs features.

## Core features

### 1. Codelab management

#### Create codelab
- Enter title, description, author
- Auto-generate unique ID
- Auto-record creation time
- Public/private visibility
- Quiz/feedback requirement settings
- Store prep guide

#### Step management
- Markdown-based content
- Automatic step ordering
- Live preview
- Bulk save (all steps)

#### Import/Export
- Export as ZIP
- Import into other systems
- Easy versioning (Git, etc.)

#### AI codelab generator
- Standard mode: quick generation
- Pro mode: plan -> draft -> review -> apply workflow
- Diff view and per-step result inspection

### 2. Attendee management

#### Registration
- Register with name and code
- Prevent duplicate names
- Auto-issued unique ID

#### Progress tracking
- Track current step
- Real-time updates
- Monitor in dashboard

#### Attendee list
- View all attendees
- Check progress
- Display registration time

#### Completion and certificates
- Mark completion
- Verify completion certificates

### 3. Real-time interaction

#### Help requests
- Request help on a step
- Real-time admin alerts
- Status management (pending/resolved)
- Timestamped requests

#### Chat
- Global chat
- 1:1 DM
- Real-time messages
- Chat history stored

#### WebSocket
- Bi-directional real-time communication
- Auto reconnect
- Connection status

### 4. Content authoring

#### Markdown support
- Standard Markdown syntax
- Code highlighting
- Image embedding
- Tables, lists, etc.

#### Code blocks
```rust
// Rust
fn main() {
    println!("Hello, world!");
}
```

```javascript
// JavaScript
console.log("Hello, world!");
```

```python
# Python
print("Hello, world!")
```

#### Image upload
- Drag and drop
- File selection
- Auto-generated URL
- Automatic WebP conversion

#### Materials
- Register links/files
- Upload material files

### 5. Learning assessment and submissions

#### Quizzes
- Multiple choice / free response
- Submit and aggregate responses

#### Assignments
- File submission
- Manage submissions

### 6. Feedback system

#### Collect feedback
- Difficulty rating (1-5)
- Satisfaction rating (1-5)
- Free-form comments
- Anonymous submission

#### Feedback analysis
- Average difficulty
- Average satisfaction
- View all responses
- Stats visualization (future)

## Admin features

### Dashboard

#### Codelab list
- View all created codelabs
- Card-style UI
- Sort by creation time
- Quick access

#### Codelab editing
- Edit metadata
- Add/edit/delete steps
- Live preview
- Auto save

#### Prep guide generation
- AI-based prep guide generation
- View step-by-step results in Pro mode

#### Attendee monitoring
- Live attendee list
- Progress display
- Identify stuck attendees
- Help request alerts

### Help request management

#### Request list
- View all requests
- Prioritize pending requests
- Show step number
- Show request time

#### Request handling
- Provide help via 1:1 DM
- Mark as resolved
- Record resolution time

### Chat management

#### Announcements
- Broadcast to all attendees
- Emphasize important announcements
- Review chat history

#### 1:1 support
- DM specific attendees
- Answer questions
- Check progress

### Export/Import

#### Export
- Export full codelab
- Download ZIP
- Backup purpose
- Migrate to other systems

#### Import
- Upload ZIP
- Auto parse
- Create new codelab
- Auto-create steps

### Audit logs
- Track admin/attendee events
- Filter by action or codelab

## Attendee features

### Select a codelab

#### List view
- Available codelab list
- Title/description/author
- Card-style UI

#### Register
- Enter name
- Enter attendee code
- Uniqueness validation
- Start immediately

### Learning flow

#### Step navigation
- Next/previous step
- Progress indicator
- Step list sidebar
- Quick jump

#### View content
- Markdown rendering
- Code highlighting
- Images
- Responsive design

#### Save progress
- Auto-save current step
- Persist after refresh
- Sync across devices

#### AI questions
- Per-step Q&A
- History stored

### Help requests

#### Create requests
- Click to request
- Current step auto attached
- Admin notified

#### Status
- Pending / resolved
- Estimated wait time
- DM notifications

### Chat participation

#### Global chat
- Talk with other attendees
- Ask/answer questions
- Real-time updates

#### Receive DMs
- Messages from admin
- Per-user alerts
- History

### Submit feedback

#### Post-completion
- Shown after last step
- Difficulty rating
- Satisfaction rating
- Comment input

## System features

### Data persistence

#### Auto-save
- Save changes immediately
- Transaction safety
- Data integrity

#### Relations
- Foreign key constraints
- Cascade deletes
- Referential integrity

### Real-time sync

#### WebSocket
- Bi-directional communication
- Low latency
- Auto reconnect

#### Event broadcast
- Attendee join/leave
- Progress updates
- Chat messages
- Help requests

### Security

#### Input validation
- Prevent SQL injection
- XSS prevention (DOMPurify)
- File upload validation
- Size limits

#### Auth and authorization
- Admin auth
- Attendee identity
- Role-based access

## Constraints

### Current limitations

1. **Single admin**: only one facilitator
2. **SQLite**: limited concurrent writes
3. **File storage**: local filesystem only
4. **WebSocket**: single server

### Future improvements

1. **Multi-admin**: multiple facilitators
2. **PostgreSQL**: scale-up support
3. **S3 storage**: cloud file management
4. **Redis**: distributed WebSocket

## Performance characteristics

### Response times
- API: < 100ms (avg ~50ms)
- Page load: < 2s
- WebSocket latency: < 50ms

### Scalability
- Concurrent users: 100 (stable)
- Max tested: 200
- Codelabs: unlimited
- Steps: unlimited

### Resource usage
- Memory: ~200MB
- CPU: ~10% (idle)
- Disk: SQLite DB + images

## Next steps

- [Database schema](database-schema.md) - DB structure
- [API reference](api-reference.md) - REST API
- [Architecture](../architecture/system-architecture.md) - system design
