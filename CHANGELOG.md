# 📝 Changelog - RedisGate

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### 🚧 In Progress
- Real Redis connection for instances (replacing simulation mode)
- API Key management UI
- Comprehensive testing suite
- Instance metrics and monitoring

---

## [0.1.0] - 2025-11-19

### ✨ Added

#### Core Features
- **Authentication System**
  - User registration with email validation
  - Login with JWT tokens (24h expiration)
  - Password hashing using bcrypt (cost factor 10)
  - Session management
  - Auto-logout on token expiration

- **Organization Management**
  - Create, read, update, delete organizations
  - Auto-create organization on first user login
  - Organization memberships with roles (owner, admin, member)
  - Multi-tenancy support

- **Redis Instance Management**
  - Create Redis instances with custom configuration
  - List instances per organization
  - Delete instances (soft delete)
  - Instance metadata storage (memory, version, etc.)
  - Slug-based instance naming

- **Quota System**
  - Track instance count per organization
  - Track memory usage per organization
  - Track API key count per organization
  - Automatic quota updates via PostgreSQL trigger
  - Enforce limits on instance/key creation
  - User-friendly quota exceeded messages

- **API Key System**
  - Generate JWT-based API keys
  - Scope-based permissions (read, write, delete)
  - API key validation middleware
  - Key expiration support
  - Auto-generate API key on login

- **Redis Command Support**
  - PING - Test connection
  - SET/GET - String operations
  - INCR - Increment counter
  - HSET/HGET - Hash operations
  - Simulation mode fallback when Redis unavailable

#### Dashboard UI
- Modern, responsive dashboard
- Landing page with feature showcase
- Login/Register pages with demo credentials
- Main dashboard with:
  - Instance list and creation
  - Organization overview
  - Quota usage display
  - Toast notifications (success/error/warning/info)
  - Auto-organization setup
- Redis command tester

#### Infrastructure
- Docker Compose setup (PostgreSQL + Redis)
- 11 database migrations
- SQLx for compile-time query verification
- Axum web framework
- CORS middleware
- Request logging
- Error handling middleware

#### Documentation
- Comprehensive README with quick start
- Quick Start guide
- Development guide
- Troubleshooting documentation
- API endpoint documentation
- Status tracking document
- This changelog!

### 🐛 Fixed
- Missing `instance_quotas` table (critical fix)
- Organization ID not stored in localStorage
- Dashboard initialization errors
- Database connection issues on Windows
- Port conflict handling
- Migration execution order

### 🔄 Changed
- Improved error messages throughout
- Better loading states in dashboard
- Enhanced toast notification styling
- Optimized database queries with indexes
- Improved startup script (start-clean.bat)

### 🗑️ Removed
- Deprecated initialization SQL scripts
- Unused test files
- Redundant migration files

---

## [0.0.1] - 2025-11-01

### ✨ Initial Release

#### Core Infrastructure
- Basic Rust project setup
- Axum web server
- PostgreSQL database connection
- Docker configuration
- Basic authentication (no JWT yet)

#### Database Schema
- Users table
- Organizations table
- Redis instances table
- API keys table
- Audit logs table

#### Basic Features
- User registration (no validation)
- Simple login
- Create Redis instances (metadata only)
- Basic frontend (HTML/CSS/JS)

---

## Version History

| Version | Date | Status | Notes |
|---------|------|--------|-------|
| 0.1.0 | 2025-11-19 | 🟢 Current | Phase 1 - 70% complete |
| 0.0.1 | 2025-11-01 | 🔴 Deprecated | Initial prototype |

---

## Upcoming Features

### v0.2.0 (Phase 1 Complete) - ETA: Early December 2025
- [ ] Real Redis connections (no more simulation)
- [ ] Complete API Key UI
- [ ] Instance list real-time updates
- [ ] Comprehensive error handling
- [ ] Basic integration tests
- [ ] Health check endpoint
- [ ] Performance optimizations

### v0.3.0 (Production Ready) - ETA: Late December 2025
- [ ] Kubernetes deployment
- [ ] Instance monitoring & metrics
- [ ] Automated backups
- [ ] Rate limiting
- [ ] Security audit
- [ ] Load testing
- [ ] Production documentation

### v1.0.0 (Public Release) - ETA: Q1 2026
- [ ] Multi-region support
- [ ] Auto-scaling
- [ ] Advanced analytics
- [ ] Billing system
- [ ] Team collaboration features
- [ ] SSO integration
- [ ] Public API documentation

---

## Migration Guide

### From 0.0.1 to 0.1.0

**Database Changes:**
```bash
# Backup your data first!
docker exec redisgate-postgres pg_dump -U redisgate_dev redisgate_dev > backup.sql

# Run new migrations
cargo run  # Migrations auto-run

# Verify
docker exec -it redisgate-postgres psql -U redisgate_dev -d redisgate_dev -c "\dt"
```

**Breaking Changes:**
- API key format changed from simple tokens to JWT
- Organization ID now required for instance operations
- Redis command endpoints changed from `/redis/command` to `/redis/:id/command`

**Environment Variables:**
```bash
# New required variables in .env
JWT_SECRET=your_secret_here  # Must be 256+ bits
```

**Frontend:**
- Clear localStorage: `localStorage.clear()`
- Users need to re-login
- Organizations auto-created on login

---

## Bug Fixes by Version

### v0.1.0
- Fixed: Address already in use error
- Fixed: Database role not found
- Fixed: Organization undefined in dashboard
- Fixed: Instance quota not updating
- Fixed: Migration execution failures
- Fixed: CORS issues with frontend
- Fixed: JWT token not persisting
- Fixed: Dashboard not showing instances
- Fixed: Error messages not user-friendly

### v0.0.1
- Fixed: Initial setup issues
- Fixed: Docker network configuration
- Fixed: Database connection pooling

---

## Performance Improvements

### v0.1.0
- Database queries optimized with indexes
- Connection pooling configured (max 5)
- Frontend toast notifications use CSS animations (no JS)
- Lazy loading for instance list
- Pagination support for large datasets

---

## Security Updates

### v0.1.0
- ✅ JWT tokens with expiration
- ✅ Password hashing with bcrypt
- ✅ API key validation middleware
- ✅ CORS configuration
- ✅ SQL injection protection (SQLx prepared statements)
- ⚠️ JWT secret should be rotated (production)
- ⚠️ Rate limiting not implemented (future)
- ⚠️ API keys not encrypted at rest (future)

---

## Contributors

### v0.1.0
- **Development Team** - Core implementation
- **GitHub Copilot** - Code assistance and documentation
- **Community** - Issue reporting and feedback

---

## Notes

### Known Issues
- Redis instances work in simulation mode only
- Dashboard may need refresh to show new instances
- API key UI incomplete
- No real-time updates
- Limited test coverage

### Deprecated Features
None yet - all features are new in v0.1.0

### Experimental Features
- Kubernetes integration (partial)
- Redis command simulation mode

---

**Keep this file updated with every release!**

For detailed changes, see: [GitHub Commits](https://github.com/yourusername/redisgate/commits/main)

---

**Format**: [Keep a Changelog](https://keepachangelog.com/)  
**Versioning**: [Semantic Versioning](https://semver.org/)  
**Last Updated**: November 19, 2025

