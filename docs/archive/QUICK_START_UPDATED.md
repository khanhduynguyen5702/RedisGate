# 🚀 RedisGate - Quick Start Guide (Updated November 19, 2025)

## ✅ What's Been Fixed

### Critical Issues Resolved:
1. ✅ **Missing Database Table** - Created `instance_quotas` table for quota tracking
2. ✅ **Dashboard Organization Issue** - Auto-creates organization on first login
3. ✅ **Toast Notifications** - Professional notification system added
4. ✅ **Error Handling** - Improved error messages throughout
5. ✅ **Clean Startup Script** - Better process management

### New Features:
- 📊 Automatic organization creation for new users
- 🔔 Toast notification system (success/error/warning/info)
- 🔄 Better error handling with user-friendly messages
- 🛠️ Clean startup script that kills old processes
- 📝 Comprehensive documentation (PHASE1_IMPROVEMENTS.md, PHASE1_SUMMARY.md)

---

## 🏃 How to Run the Project

### Option 1: Using the Clean Startup Script (Recommended)

```batch
.\start-clean.bat
```

This will:
1. Kill any existing RedisGate processes
2. Check Docker is running
3. Start PostgreSQL and Redis containers
4. Run database migrations
5. Start the RedisGate server

### Option 2: Manual Steps

```powershell
# 1. Start Docker services
docker start redisgate-postgres redisgate-redis

# Or if containers don't exist:
docker-compose up -d

# 2. Verify services are running
docker ps

# 3. Kill any existing processes
taskkill /F /IM redisgate.exe 2>nul

# 4. Start the server
cargo run --bin redisgate
```

### Option 3: Development Mode

```powershell
# Start services only (if you want to run server from IDE)
docker-compose up -d

# In your IDE, run the 'redisgate' binary
```

---

## 🧪 Testing the Application

### 1. Access the UI
Open browser to: **http://localhost:3000**

### 2. Login with Demo Account
- **Email:** `demo@redisgate.dev`
- **Password:** `Demo123456!`

### 3. Create a Redis Instance
1. Dashboard should load automatically
2. Click "Create Instance" button
3. Fill in:
   - **Name:** `My Test Instance`
   - **Memory:** `512 MB` (default)
   - **Version:** `7.0` (default)
4. Click "Create Instance"
5. Instance should appear in dashboard

### 4. Test Redis Commands
In the dashboard, find your instance and try:
- **PING** - Should return "PONG"
- **SET key value** - Store a value
- **GET key** - Retrieve the value

---

## 🐛 Troubleshooting

### "Address already in use" Error

**Problem:** Port 3000 is already in use

**Solution:**
```powershell
# Find and kill the process
netstat -ano | findstr :3000
taskkill /F /PID <process_id>

# Or use the clean startup script
.\start-clean.bat
```

### "Docker is not running" Error

**Problem:** Docker Desktop is not started

**Solution:**
1. Start Docker Desktop
2. Wait for it to fully start (whale icon in system tray)
3. Run the startup script again

### "Role redisgate_dev does not exist" Error

**Problem:** Database user not created

**Solution:**
```powershell
# Create the database role
docker exec -it redisgate-postgres psql -U postgres -c "CREATE ROLE redisgate_dev WITH LOGIN PASSWORD 'redisgate_dev_password';"
docker exec -it redisgate-postgres psql -U postgres -c "CREATE DATABASE redisgate_dev OWNER redisgate_dev;"

# Restart the server
cargo run --bin redisgate
```

### "Cannot connect to Redis instance" Error

**Problem:** Created instances show "simulation mode"

**Current Status:** This is expected behavior in development
- Redis instances are created in the database
- Connection to actual Redis pods requires Kubernetes
- For local development, commands are simulated

**To Fix (Future):**
- Set up Kubernetes (Minikube or Docker Desktop Kubernetes)
- Or modify code to use local Redis container

### Dashboard Shows No Instances

**Problem:** Instances don't appear after creation

**Debug Steps:**
1. Open browser console (F12)
2. Look for errors in console
3. Check Network tab for failed API calls
4. Verify organization ID is set: `localStorage.getItem('organizationId')`

**Solution:**
```javascript
// In browser console
localStorage.clear()  // Clear all data
// Then login again
```

---

## 📂 Project Structure

```
RedisGate/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── handlers/               # API route handlers
│   │   ├── auth.rs            # Authentication (login/register)
│   │   ├── redis.rs           # Redis command handlers
│   │   ├── redis_instances.rs # Instance management
│   │   ├── api_keys.rs        # API key management
│   │   └── organizations.rs   # Organization management
│   ├── services/
│   │   └── quota.rs           # Quota checking service
│   ├── models.rs              # Database models
│   ├── auth.rs                # JWT authentication
│   └── middleware.rs          # Request middleware
├── migrations/                 # Database migrations
│   └── 202511190000001_create_instance_quotas_table.sql
├── public/                    # Frontend files
│   ├── index.html            # Landing page
│   ├── login.html            # Login/register page
│   └── dashboard.html        # Main dashboard (UPDATED)
├── docker-compose.yml         # Docker services config
├── .env.development          # Development environment
├── start-clean.bat           # Clean startup script (NEW)
├── PHASE1_IMPROVEMENTS.md    # Improvement plan (NEW)
├── PHASE1_SUMMARY.md         # Progress summary (NEW)
└── README.md                 # This file
```

---

## 🔑 API Endpoints

### Authentication
- `POST /auth/register` - Register new user
- `POST /auth/login` - Login and get JWT token
- `GET /auth/me` - Get current user info

### Organizations
- `GET /api/organizations` - List user's organizations
- `POST /api/organizations` - Create organization
- `GET /api/organizations/:id` - Get organization details
- `PUT /api/organizations/:id` - Update organization
- `DELETE /api/organizations/:id` - Delete organization

### Redis Instances
- `GET /api/organizations/:org_id/redis-instances` - List instances
- `POST /api/organizations/:org_id/redis-instances` - Create instance
- `GET /api/organizations/:org_id/redis-instances/:id` - Get instance
- `DELETE /api/organizations/:org_id/redis-instances/:id` - Delete instance

### Redis Commands
- `GET /redis/:instance_id/ping` - Test connection
- `GET /redis/:instance_id/set/:key/:value` - Set a value
- `GET /redis/:instance_id/get/:key` - Get a value
- `GET /redis/:instance_id/incr/:key` - Increment a counter
- `GET /redis/:instance_id/hset/:key/:field/:value` - Set hash field
- `GET /redis/:instance_id/hget/:key/:field` - Get hash field

### Quota
- `GET /api/organizations/:id/quota` - Get quota info

---

## 💾 Database Schema

### Key Tables:
- `users` - User accounts
- `organizations` - Organizations/workspaces
- `organization_memberships` - User-org relationships
- `redis_instances` - Redis instance metadata
- `api_keys` - API keys for Redis access
- `instance_quotas` - ✨ NEW: Quota tracking per organization
- `audit_logs` - Activity logging

---

## 🎯 Current Limitations

### Known Issues:
1. **Redis Connection** - Instances work in simulation mode only
   - Real Redis connection requires Kubernetes setup
   - Local Redis container exists but instances don't connect to it yet

2. **Instance Display** - May need browser refresh to see newly created instances
   - Auto-refresh not fully working
   - Workaround: Refresh page after creating instance

3. **API Key UI** - Backend exists but UI incomplete
   - Can create keys via API
   - Dashboard UI needs completion

4. **No Real-time Updates** - Dashboard doesn't auto-refresh
   - Must manually refresh to see changes
   - Needs WebSocket or polling implementation

### Security Notes:
- JWT secret is hardcoded in `.env.development` (CHANGE IN PRODUCTION)
- API keys stored as JWT tokens (plaintext in DB)
- No rate limiting implemented yet
- CORS is permissive (lock down for production)

---

## 🚀 Next Steps for Development

### High Priority:
1. **Fix Redis Connections** - Make instances connect to actual Redis
2. **Complete API Key UI** - Finish the dashboard implementation
3. **Add Real-time Updates** - WebSocket or polling for dashboard
4. **Improve Error Messages** - More user-friendly messages

### Medium Priority:
5. **Add Tests** - Unit, integration, and E2E tests
6. **Health Check Endpoint** - `/health` for monitoring
7. **Metrics** - Prometheus-compatible metrics
8. **Rate Limiting** - Prevent API abuse

### Future Features:
9. **Kubernetes Integration** - Full K8s deployment
10. **Backup/Restore** - Redis data backup
11. **Multi-region** - Deploy instances in different regions
12. **Billing** - Usage tracking and billing integration

---

## 📚 Additional Documentation

- **PHASE1_IMPROVEMENTS.md** - Comprehensive improvement plan
- **PHASE1_SUMMARY.md** - Current progress and status
- **migrations/README.md** - Database migration guide

---

## 🤝 Contributing

### Before Making Changes:
1. Read PHASE1_IMPROVEMENTS.md to understand the roadmap
2. Check PHASE1_SUMMARY.md for current status
3. Test your changes with the demo account
4. Update documentation as needed

### Testing Checklist:
- [ ] Can register new user
- [ ] Can login with demo account
- [ ] Organization auto-created on login
- [ ] Can create Redis instance
- [ ] Instance appears in dashboard
- [ ] Can execute PING command
- [ ] Toast notifications work
- [ ] Error messages are clear

---

## 📞 Support

**Getting Stuck?**
1. Check this README's troubleshooting section
2. Review PHASE1_SUMMARY.md for known issues
3. Check browser console for errors (F12)
4. Check server logs for backend errors

**Common Issues:**
- Port already in use → Use `start-clean.bat`
- Docker not running → Start Docker Desktop
- Database errors → Check PostgreSQL container is running
- Frontend errors → Clear localStorage and try again

---

## ✨ Features Showcase

### What Works Now:
✅ User registration and authentication  
✅ JWT-based session management  
✅ Automatic organization creation  
✅ Redis instance creation (database level)  
✅ Quota tracking and enforcement  
✅ Professional toast notifications  
✅ Dashboard with instance list  
✅ API key generation (backend)  

### Coming Soon:
⏳ Real Redis connections  
⏳ Complete API key UI  
⏳ Real-time dashboard updates  
⏳ Instance metrics and monitoring  
⏳ Backup and restore  
⏳ Kubernetes deployment  

---

## 🎓 Learning Resources

### Understanding the Codebase:
- **Axum Framework** - Web framework used
- **SQLx** - Type-safe SQL queries
- **JWT** - Token-based authentication
- **Redis** - In-memory data store

### Key Concepts:
- **Organization** - Workspace that contains instances and users
- **Instance** - A Redis database deployment
- **API Key** - JWT token for Redis command authentication
- **Quota** - Resource limits per organization

---

**Last Updated:** November 19, 2025  
**Version:** 0.1.0 (Phase 1 In Progress)  
**Status:** 🟡 Development - Core features working, polish needed

---

**Pro Tip:** Use the demo account (`demo@redisgate.dev` / `Demo123456!`) for quick testing. It has a pre-configured organization and sample data.

