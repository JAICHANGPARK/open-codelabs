# API Usage Examples

This document introduces examples of how to use the REST API.

## Codelab API

### List Codelabs

```bash
curl http://localhost:8080/api/codelabs
```

```javascript
const codelabs = await fetch('http://localhost:8080/api/codelabs')
    .then(res => res.json());
```

### Create Codelab

```bash
curl -X POST http://localhost:8080/api/codelabs \
  -H "Content-Type: application/json" \
  -d '{"title":"Test","description":"Desc","author":"Me"}'
```

```javascript
const codelab = await fetch('http://localhost:8080/api/codelabs', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
        title: 'Test',
        description: 'Description',
        author: 'Author'
    })
}).then(res => res.json());
```

## WebSocket

```javascript
const ws = new WebSocket('ws://localhost:8080/api/ws/codelab_id');

ws.onopen = () => {
    ws.send(JSON.stringify({
        type: 'join',
        name: 'John Doe'
    }));
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log('Received:', data);
};
```

## Next Steps

- [API Reference](../specification/api-reference.md)
- [Backend Examples](backend-examples.md)
