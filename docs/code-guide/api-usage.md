# API 사용 예제

REST API 사용 예제를 소개합니다.

## Codelab API

### 목록 조회

```bash
curl http://localhost:8080/api/codelabs
```

```javascript
const codelabs = await fetch('http://localhost:8080/api/codelabs')
    .then(res => res.json());
```

### 생성

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
        name: '홍길동'
    }));
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log('Received:', data);
};
```

## 다음 단계

- [API 레퍼런스](../specification/api-reference.md)
- [Backend 예제](backend-examples.md)
