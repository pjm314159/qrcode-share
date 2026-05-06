# API Documentation

## Overview

Base URL: `http://localhost:3000`

All API endpoints return JSON. WebSocket is used for real-time message delivery.

### Common Response Format

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "CHANNEL_NOT_FOUND",
    "message": "Channel with id 'abc' does not exist"
  }
}
```

### HTTP Status Codes

| Status Code | Meaning |
|-------------|---------|
| 200 | Success |
| 201 | Created |
| 400 | Bad Request (validation error) |
| 401 | Unauthorized (password required) |
| 403 | Forbidden (wrong password) |
| 404 | Not Found |
| 429 | Too Many Requests (rate limited) |
| 500 | Internal Server Error |
| 503 | Service Unavailable (server overloaded) |

---

## Channel API

### 1. Create Channel

Create a new channel.

```
POST /api/channels
```

#### Request Body

```json
{
  "name": "Math Class Sign-in",
  "password": "optional_password",
  "link_limitation": ["dingtalk.com", "weixin.qq.com"],
  "channel_type": "sign-in",
  "location": "Building A Room 301",
  "teacher": "Prof. Zhang"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | Yes | Channel display name (max 100 chars) |
| password | string | No | Channel access password (max 64 chars) |
| link_limitation | string[] | No | Allowed QR code link domains for security |
| channel_type | string | No | Channel type tag (e.g., "sign-in", "payment") |
| location | string | No | Physical location description |
| teacher | string | No | Teacher or organizer name |

#### Response (201)

```json
{
  "success": true,
  "data": {
    "id": "a1b2c3d4",
    "name": "Math Class Sign-in",
    "has_password": true,
    "link_limitation": ["dingtalk.com", "weixin.qq.com"],
    "channel_type": "sign-in",
    "location": "Building A Room 301",
    "teacher": "Prof. Zhang",
    "created_at": "2026-04-30T10:00:00Z",
    "subscriber_count": 0,
    "message_count": 0
  }
}
```

#### Error Responses

| Code | HTTP | Condition |
|------|------|-----------|
| CHANNEL_LIMIT_REACHED | 429 | Global channel limit (5000) reached |
| USER_CHANNEL_LIMIT | 429 | Per-user channel limit (10) reached |
| VALIDATION_ERROR | 400 | Invalid input data |

---

### 2. Get Channel Info

Get channel details by ID.

```
GET /api/channels/:channel_id
```

#### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| channel_id | string | Channel unique ID |

#### Query Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| password | string | Conditional | Required if channel has password |

#### Response (200)

```json
{
  "success": true,
  "data": {
    "id": "a1b2c3d4",
    "name": "Math Class Sign-in",
    "has_password": true,
    "link_limitation": ["dingtalk.com", "weixin.qq.com"],
    "channel_type": "sign-in",
    "location": "Building A Room 301",
    "teacher": "Prof. Zhang",
    "created_at": "2026-04-30T10:00:00Z",
    "subscriber_count": 15,
    "message_count": 42
  }
}
```

#### Error Responses

| Code | HTTP | Condition |
|------|------|-----------|
| CHANNEL_NOT_FOUND | 404 | Channel does not exist |
| PASSWORD_REQUIRED | 401 | Channel requires password |
| WRONG_PASSWORD | 403 | Incorrect password |

---

### 3. List Channels

List active channels with optional filtering.

```
GET /api/channels
```

#### Query Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| page | integer | No | 1 | Page number |
| limit | integer | No | 20 | Items per page (max 100) |
| channel_type | string | No | - | Filter by channel type |
| search | string | No | - | Search by channel name |

#### Response (200)

```json
{
  "success": true,
  "data": {
    "channels": [
      {
        "id": "a1b2c3d4",
        "name": "Math Class Sign-in",
        "has_password": true,
        "link_limitation": ["dingtalk.com", "weixin.qq.com"],
        "channel_type": "sign-in",
        "location": "Building A Room 301",
        "teacher": "Prof. Zhang",
        "created_at": "2026-04-30T10:00:00Z",
        "subscriber_count": 15,
        "message_count": 42
      }
    ],
    "total": 85,
    "page": 1,
    "limit": 20
  }
}
```

---

### 4. Update Channel

Update channel settings.

```
PATCH /api/channels/:channel_id
```

#### Request Body

```json
{
  "name": "Updated Channel Name",
  "password": "new_password",
  "link_limitation": ["dingtalk.com"],
  "channel_type": "lecture",
  "location": "Building B Room 201",
  "teacher": "Dr. Smith"
}
```

All fields are optional. Only provided fields will be updated.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | No | Channel display name (max 100 chars) |
| password | string | No | Channel access password |
| link_limitation | string[] | No | Allowed link domains |
| channel_type | string | No | Channel type tag |
| location | string | No | Location description |
| teacher | string | No | Teacher/organizer name |

#### Response (200)

```json
{
  "success": true,
  "data": {
    "id": "a1b2c3d4",
    "name": "Updated Channel Name",
    "has_password": true,
    "link_limitation": ["dingtalk.com"],
    "channel_type": "lecture",
    "location": "Building B Room 201",
    "teacher": "Dr. Smith",
    "created_at": "2026-04-30T10:00:00Z",
    "subscriber_count": 15,
    "message_count": 42
  }
}
```

---

### 5. Delete Channel

Delete a channel and all its messages.

```
DELETE /api/channels/:channel_id
```

#### Response (200)

```json
{
  "success": true,
  "data": {
    "deleted": true
  }
}
```

---

## Message API

### 1. Send Message

Send a QR code link message to a channel.

```
POST /api/channels/:channel_id/messages
```

#### Request Body

```json
{
  "name": "DingTalk Sign-in",
  "link": "https://dingtalk.com/sign-in/abc123",
  "message_type": "sign-in",
  "location": "Building A",
  "expire_seconds": 3600
}
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| name | string | Yes | - | Message display name (max 100 chars) |
| link | string | Yes | - | QR code URL (must start with http:// or https://) |
| message_type | string | No | null | Message type tag |
| location | string | No | null | Location description |
| expire_seconds | integer | No | 3600 | TTL in seconds (1-3600) |

#### Validation Rules

- Total message size must not exceed 5KB
- `link` must be a valid HTTP/HTTPS URL
- If channel has `link_limitation`, the link domain must match one of the allowed domains
- Rate limit: 60 messages per minute per user

#### Response (201)

```json
{
  "success": true,
  "data": {
    "id": "msg_abc123",
    "name": "DingTalk Sign-in",
    "link": "https://dingtalk.com/sign-in/abc123",
    "link_domain": "dingtalk.com",
    "message_type": "sign-in",
    "location": "Building A",
    "expire_at": "2026-04-30T11:00:00Z",
    "created_at": "2026-04-30T10:00:00Z"
  }
}
```

#### Error Responses

| Code | HTTP | Condition |
|------|------|-----------|
| CHANNEL_NOT_FOUND | 404 | Channel does not exist |
| MESSAGE_TOO_LARGE | 400 | Total size exceeds 5KB |
| INVALID_LINK_FORMAT | 400 | Link is not a valid HTTP/HTTPS URL |
| LINK_DOMAIN_NOT_ALLOWED | 400 | Link domain not in channel's allowed list |
| RATE_LIMIT_EXCEEDED | 429 | Too many messages |
| CHANNEL_MESSAGE_LIMIT | 429 | Channel message limit (300) reached |

---

### 2. Get Channel Messages

Get recent messages in a channel.

```
GET /api/channels/:channel_id/messages
```

#### Query Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| limit | integer | No | 50 | Messages to return (max 100) |
| cursor | string | No | - | Message ID cursor for pagination |

#### Response (200)

```json
{
  "success": true,
  "data": {
    "messages": [
      {
        "id": "msg_abc123",
        "name": "DingTalk Sign-in",
        "link": "https://dingtalk.com/sign-in/abc123",
        "link_domain": "dingtalk.com",
        "message_type": "sign-in",
        "location": "Building A",
        "expire_at": "2026-04-30T11:00:00Z",
        "created_at": "2026-04-30T10:00:00Z"
      }
    ],
    "has_more": false,
    "next_cursor": null
  }
}
```

---

### 3. Get Single Message

Get a specific message by ID.

```
GET /api/channels/:channel_id/messages/:message_id
```

#### Response (200)

```json
{
  "success": true,
  "data": {
    "id": "msg_abc123",
    "name": "DingTalk Sign-in",
    "link": "https://dingtalk.com/sign-in/abc123",
    "link_domain": "dingtalk.com",
    "message_type": "sign-in",
    "location": "Building A",
    "expire_at": "2026-04-30T11:00:00Z",
    "created_at": "2026-04-30T10:00:00Z"
  }
}
```

#### Error Responses

| Code | HTTP | Condition |
|------|------|-----------|
| MESSAGE_NOT_FOUND | 404 | Message does not exist or expired |
| CHANNEL_NOT_FOUND | 404 | Channel does not exist |

---

## WebSocket API

### 1. Connect to Channel

Establish a WebSocket connection to receive real-time messages.

```
WS /api/channels/:channel_id/ws
```

#### Query Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| password | string | Conditional | Required if channel has password |

#### Connection Flow

```
Client                                          Server
  |                                               |
  |--- WS Connect /api/channels/a1b2c3d4/ws ---->|
  |                                               |
  |<-- Connection Established -------------------->|
  |                                               |
  |<-- {"type":"connected","channel_id":"a1b2c3d4","subscriber_count":15} |
  |                                               |
  |<-- {"type":"message","id":"msg_abc123",...} --|  (broadcast)
  |<-- {"type":"message","id":"msg_def456",...} --|  (broadcast)
  |                                               |
  |--- {"type":"ping"} ------------------------->|
  |<-- {"type":"pong"} --------------------------|
  |                                               |
  |--- Close Frame ----------------------------->|
  |<-- Close Frame ------------------------------|
```

#### Server → Client Messages

##### Connected

```json
{
  "type": "connected",
  "channel_id": "a1b2c3d4",
  "subscriber_count": 15
}
```

##### New Message (Broadcast)

```json
{
  "type": "message",
  "id": "msg_abc123",
  "name": "DingTalk Sign-in",
  "link": "https://dingtalk.com/sign-in/abc123",
  "message_type": "sign-in",
  "location": "Building A",
  "created_at": 1717460400
}
```

| Field | Type | Description |
|-------|------|-------------|
| type | string | Always "message" |
| id | string | Unique message ID |
| name | string | Display name |
| link | string | QR code URL |
| message_type | string\|null | Message type tag |
| location | string\|null | Location description |
| created_at | integer | Unix timestamp (seconds) |

##### Subscriber Count Update

```json
{
  "type": "subscriber_update",
  "channel_id": "a1b2c3d4",
  "subscriber_count": 16
}
```

##### Pong

```json
{
  "type": "pong"
}
```

##### Error

```json
{
  "type": "error",
  "code": "CHANNEL_DELETED",
  "message": "Channel has been deleted"
}
```

#### Client → Server Messages

##### Ping

```json
{
  "type": "ping"
}
```

#### Error Codes

| Code | Description |
|------|-------------|
| CHANNEL_NOT_FOUND | Channel does not exist |
| CHANNEL_DELETED | Channel was deleted while connected |
| WRONG_PASSWORD | Incorrect password |
| RATE_LIMITED | Sending too many messages |
| CONNECTION_LIMIT | Max connections reached |

#### Heartbeat

- Server sends Ping every 30 seconds
- Client must respond with Pong within 60 seconds
- Connection is closed if no Pong received

#### Reconnection

Client should implement exponential backoff reconnection:

```
1st attempt: wait 1s
2nd attempt: wait 2s
3rd attempt: wait 4s
4th attempt: wait 8s
Max wait: 30s
```

---

## System API

### 1. Health Check

```
GET /health
```

#### Response (200)

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 86400,
  "checks": {
    "memory": {
      "used_mb": 500,
      "limit_mb": 2000,
      "percentage": 25.0,
      "healthy": true
    },
    "channels": {
      "count": 10,
      "limit": 5000,
      "percentage": 0.2,
      "healthy": true
    }
  }
}
```

#### Response (503) — Unhealthy

```json
{
  "status": "unhealthy",
  "version": "0.1.0",
  "uptime_seconds": 86400,
  "checks": {
    "memory": {
      "used_mb": 1900,
      "limit_mb": 2000,
      "percentage": 95.0,
      "healthy": false
    },
    "channels": {
      "count": 5000,
      "limit": 5000,
      "percentage": 100.0,
      "healthy": false
    }
  }
}
```

---

### 2. Metrics

```
GET /metrics
```

#### Response (200)

```json
{
  "channels": {
    "total": 42,
    "active": 38
  },
  "messages": {
    "total": 15234,
    "per_channel_avg": 362.7
  },
  "connections": {
    "active_websocket": 87,
    "total_subscribers": 150
  },
  "system": {
    "memory_used_mb": 500,
    "memory_limit_mb": 2000,
    "cpu_usage_percent": 25.0,
    "uptime_seconds": 86400
  }
}
```

---

## Rate Limiting

### Global Limits

| Resource | Limit | Window |
|----------|-------|--------|
| Channel creation | 10 per user | - |
| Message sending | 60 per user | 1 minute |
| WebSocket connections | 500 total | - |
| Connections per channel | 50 | - |

### Rate Limit Headers

Every response includes rate limit headers:

```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1717460460
```

### Rate Limit Exceeded Response (429)

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Please try again later.",
    "retry_after_seconds": 15
  }
}
```

---

## Error Codes Reference

| Code | HTTP | Description |
|------|------|-------------|
| VALIDATION_ERROR | 400 | Invalid request data |
| INVALID_LINK_FORMAT | 400 | Link is not a valid HTTP/HTTPS URL |
| LINK_DOMAIN_NOT_ALLOWED | 400 | Link domain not in channel's allowed list |
| MESSAGE_TOO_LARGE | 400 | Message exceeds 5KB |
| PASSWORD_REQUIRED | 401 | Channel requires password |
| WRONG_PASSWORD | 403 | Incorrect channel password |
| CHANNEL_NOT_FOUND | 404 | Channel does not exist |
| MESSAGE_NOT_FOUND | 404 | Message does not exist or expired |
| RATE_LIMIT_EXCEEDED | 429 | Too many requests |
| CHANNEL_LIMIT_REACHED | 429 | Global channel limit reached |
| CHANNEL_MESSAGE_LIMIT | 429 | Channel message limit reached |
| USER_CHANNEL_LIMIT | 429 | User channel creation limit reached |
| CONNECTION_LIMIT | 429 | WebSocket connection limit reached |
| DATABASE_ERROR | 500 | Database operation failed |
| SERVER_OVERLOADED | 503 | Server is temporarily overloaded |

---

## Data Models

### Channel

```typescript
interface Channel {
  id: string;                  // Unique channel ID (8 chars)
  name: string;                // Display name
  has_password: boolean;       // Whether channel has password
  link_limitation?: string[];  // Allowed link domains
  channel_type?: string;       // Channel type tag
  location?: string;           // Location description
  teacher?: string;            // Teacher/organizer name
  created_at: string;          // ISO 8601 timestamp
  subscriber_count: number;    // Current WebSocket subscribers
  message_count: number;       // Current message count
}
```

### Message

```typescript
interface Message {
  id: string;                  // Unique message ID
  name: string;                // Display name
  link: string;                // QR code URL
  link_domain: string;         // Extracted domain from link
  message_type?: string;       // Message type tag
  location?: string;           // Location description
  expire_at: string;           // ISO 8601 expiration time
  created_at: string;          // ISO 8601 creation time
}
```

### WebSocket Broadcast Message

```typescript
interface WsBroadcastMessage {
  type: "message";             // Message type identifier
  id: string;                  // Unique message ID
  name: string;                // Display name
  link: string;                // QR code URL
  message_type?: string;       // Message type tag
  location?: string;           // Location description
  created_at: number;          // Unix timestamp (seconds)
}
```

### Message Card (Frontend Display)

```typescript
interface MessageCard {
  name: string;                // Display name
  expire_at: string;           // With countdown timer
  message_type?: string;       // Type badge
  link_domain: string;         // Domain display only
}
```
