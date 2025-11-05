# API 调用示例

## 运行后端

### 1. 准备数据库

确保 PostgreSQL 已安装并运行，创建数据库：

```sql
CREATE DATABASE personal_finance_tracker;
```

### 2. 配置环境变量

创建 `.env` 文件（复制 `.env.example`）：

```bash
cp .env.example .env
```

编辑 `.env` 文件，设置正确的数据库连接：

```
DATABASE_URL=postgresql://用户名:密码@localhost:5432/personal_finance_tracker
```

### 3. 运行数据库迁移

```bash
# 如果安装了 sqlx-cli
sqlx migrate run

# 或者手动执行 migrations/20251102021659_init.sql 中的 SQL
```

### 4. 启动服务器

```bash
cargo run
```

服务器将在 `http://127.0.0.1:3000` 启动

---

## Postman API 调用示例

### 1. Health Check (测试服务器是否运行)

**Method:** `GET`  
**URL:** `http://127.0.0.1:3000/health`  
**Headers:** 不需要  
**Body:** 不需要

---

### 2. 注册新用户

**Method:** `POST`  
**URL:** `http://127.0.0.1:3000/api/auth/register`  
**Headers:**

```
Content-Type: application/json
```

**Body (raw JSON):**

```json
{
  "username": "testuser",
  "password": "password123"
}
```

**成功响应示例:**

```json
{
  "message": "User registered successfully",
  "user_id": 1
}
```

**错误响应示例:**

```json
{
  "error": "Username already exists"
}
```

---

### 3. 用户登录

**Method:** `POST`  
**URL:** `http://127.0.0.1:3000/api/auth/login`  
**Headers:**

```
Content-Type: application/json
```

**Body (raw JSON):**

```json
{
  "username": "testuser",
  "password": "password123"
}
```

**成功响应示例:**

```json
{
  "message": "Login successful",
  "token": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "user_id": 1
}
```

**重要:** 保存返回的 `token`，后续需要认证的 API 会用到！

**错误响应示例:**

```json
{
  "error": "Invalid username or password"
}
```

---

### 4. 用户登出

**Method:** `POST`  
**URL:** `http://127.0.0.1:3000/api/auth/logout`  
**Headers:**

```
Content-Type: application/json
Authorization: Bearer <your_token_here>
```

**注意:** 将 `<your_token_here>` 替换为登录时获得的 token

**Body:** 不需要（或发送空的 JSON `{}`）

**成功响应示例:**

```json
{
  "message": "Logout successful"
}
```

**错误响应示例（未提供 token）:**

```json
{
  "error": "Missing or invalid authorization header"
}
```

**错误响应示例（token 无效）:**

```json
{
  "error": "Invalid session token"
}
```

---

## 测试流程

### 完整测试流程：

1. **测试 Health Check**

   - GET `/health`
   - 应该返回 `{"status": "ok"}`

2. **注册用户**

   - POST `/api/auth/register`
   - 使用上面的 JSON body
   - 保存返回的 `user_id`

3. **登录**

   - POST `/api/auth/login`
   - 使用相同的用户名和密码
   - **保存返回的 `token`**

4. **测试登出（需要认证）**

   - POST `/api/auth/logout`
   - 在 Headers 中添加 `Authorization: Bearer <token>`
   - 应该成功登出

5. **再次测试登出（token 已失效）**
   - 使用相同的 token 再次尝试登出
   - 应该返回错误（token 已失效）

---

## Postman 设置提示

### 创建 Collection

1. 创建新的 Collection 命名为 "Personal Finance Tracker"
2. 在 Collection Variables 中添加：
   - `base_url`: `http://127.0.0.1:3000`
   - `token`: (登录后手动更新)

### 使用环境变量

在 Postman 中设置环境变量：

- `base_url`: `http://127.0.0.1:3000`
- `token`: (登录后从响应中复制)

然后在 URL 中使用: `{{base_url}}/api/auth/login`
在 Authorization header 中使用: `Bearer {{token}}`
