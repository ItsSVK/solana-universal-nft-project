# Universal NFT Protocol - Error Recovery & Monitoring Guide

This comprehensive guide covers the error handling, recovery, and monitoring system implemented for the Universal NFT Protocol.

## 🎯 Overview

The Error Recovery System provides enterprise-grade error handling capabilities including:
- **Timeout Management**: Automatic detection and handling of stuck cross-chain messages
- **Retry Logic**: Intelligent retry mechanisms with exponential backoff
- **Admin Recovery**: Manual intervention tools for complex failure scenarios
- **Comprehensive Monitoring**: Structured logging, metrics, and alerting

## 🏗️ Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                 Error Recovery System                       │
├─────────────────┬─────────────────┬─────────────────────────┤
│ ErrorHandling   │ ErrorRecovery   │ MonitoringLogger        │
│ Library         │ Manager         │ Contract                │
│                 │                 │                         │
│ • Timeout logic │ • Message mgmt  │ • Structured logging    │
│ • Retry calc    │ • Recovery ops  │ • Metrics collection    │
│ • Status mgmt   │ • Admin funcs   │ • Alert system         │
│ • Validation    │ • Config mgmt   │ • Audit trail          │
└─────────────────┴─────────────────┴─────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│            Enhanced Universal NFT Contracts                 │
├─────────────────────────────┬───────────────────────────────┤
│ UniversalNFTEnhanced        │ UniversalNFTReceiverEnhanced  │
│ (ZetaChain)                 │ (Base Sepolia)                │
│                             │                               │
│ • Enhanced transfers        │ • Enhanced receiving          │
│ • Automatic monitoring      │ • Error handling              │
│ • Recovery integration      │ • Monitoring integration      │
│ • Revert handling           │ • Admin functions             │
└─────────────────────────────┴───────────────────────────────┘
```

## 🔧 Quick Start

### 1. Deploy the System

```bash
# Deploy error recovery components
npm run deploy:error-recovery

# Deploy to specific network
npx hardhat run scripts/deploy-error-recovery.js --network base-sepolia
```

### 2. Configure Monitoring

```javascript
// Get deployed contracts
const errorRecoveryManager = await ethers.getContractAt(
  "ErrorRecoveryManager", 
  "YOUR_RECOVERY_MANAGER_ADDRESS"
);

// Configure retry settings
await errorRecoveryManager.updateRetryConfig(
  5,     // maxRetries
  300,   // baseDelay (5 minutes)
  14400, // maxDelay (4 hours)  
  2,     // backoffMultiplier
  true   // useExponentialBackoff
);

// Set up alerts
await monitoringLogger.createAlertRule(
  "High Error Rate",
  "operations_failed", 
  10,   // threshold
  3600, // window (1 hour)
  1800, // cooldown (30 min)
  "Manual review required"
);
```

### 3. Integration with Existing Contracts

```solidity
// In your NFT contract
import "../shared/ErrorRecoveryManager.sol";

contract YourNFTContract {
    ErrorRecoveryManager public immutable errorRecovery;
    
    constructor(address _errorRecovery) {
        errorRecovery = ErrorRecoveryManager(_errorRecovery);
    }
    
    function transferWithMonitoring(...) external {
        // Register message for monitoring
        errorRecovery.registerMessage(message, msg.sender);
        
        // Attempt transfer
        try this._executeTransfer(...) {
            // Success - mark completed
            errorRecovery.markMessageCompleted(messageId);
        } catch (Error memory err) {
            // Report error for retry
            errorRecovery.reportMessageError(
                messageId,
                "TRANSFER_ERROR", 
                err,
                true // shouldRetry
            );
        }
    }
}
```

## 📋 Subtask 1: Message Timeout Mechanism

### Features

- **Automatic Timeout Detection**: Messages are monitored for timeout based on chain-specific settings
- **Chain-Specific Timeouts**: Different timeout values for different blockchain networks
- **Configurable Windows**: Flexible timeout configuration for various scenarios

### Configuration

```javascript
// Set default timeouts
await errorRecoveryManager.updateTimeoutConfig(
  3600,  // defaultTimeout (1 hour)
  7200   // extendedTimeout (2 hours)
);

// Set chain-specific timeouts
await errorRecoveryManager.setChainTimeout(900, 7200);    // Solana: 2 hours
await errorRecoveryManager.setChainTimeout(7001, 3600);  // ZetaChain: 1 hour
await errorRecoveryManager.setChainTimeout(84532, 3600); // Base: 1 hour
```

### Usage

```javascript
// Check for timed out messages
const timedOutMessages = await errorRecoveryManager.getTimedOutMessages(
  fromTimestamp,
  toTimestamp
);

// Handle timeouts
const handledCount = await errorRecoveryManager.handleTimeouts(timedOutMessages);
console.log(`Handled ${handledCount} timeouts`);
```

### Events

```solidity
event MessageTimeout(
    bytes32 indexed messageId,
    uint32 indexed destinationChain,
    address indexed sender,
    uint256 timeoutAt,
    string reason
);
```

## 🔄 Subtask 2: Retry Logic with Exponential Backoff

### Features

- **Exponential Backoff**: Intelligent delay calculation to avoid overwhelming networks
- **Maximum Retry Limits**: Configurable limits to prevent infinite retry loops
- **Automatic Execution**: Background processing of retry queue
- **Manual Override**: Admin ability to force retries

### Configuration

```javascript
// Configure retry behavior
await errorRecoveryManager.updateRetryConfig(
  5,     // maxRetries: Maximum number of retry attempts
  300,   // baseDelay: Initial delay (5 minutes)
  14400, // maxDelay: Maximum delay (4 hours)
  2,     // backoffMultiplier: Exponential factor
  true   // exponentialBackoff: Use exponential backoff
);
```

### Retry Calculation

The retry delay is calculated as:
```
delay = min(baseDelay * (backoffMultiplier ^ retryAttempt), maxDelay)
```

Example progression with `baseDelay=300s`, `backoffMultiplier=2`:
- Attempt 1: 300s (5 minutes)
- Attempt 2: 600s (10 minutes)  
- Attempt 3: 1200s (20 minutes)
- Attempt 4: 2400s (40 minutes)
- Attempt 5: 4800s (80 minutes, capped at maxDelay)

### Usage

```javascript
// Execute automatic retries
const processedCount = await errorRecoveryManager.executeAutoRetries(10);

// Force retry specific message (admin only)
await errorRecoveryManager.forceRetry(
  messageId,
  "Manual retry requested by admin"
);

// Report error to trigger retry
await errorRecoveryManager.reportMessageError(
  messageId,
  "NETWORK_ERROR",
  "Connection timeout",
  true // shouldRetry
);
```

### Events

```solidity
event MessageRetryScheduled(
    bytes32 indexed messageId,
    uint256 indexed retryAttempt,
    uint256 nextRetryAt,
    uint256 delay,
    string reason
);

event AutoRetryExecuted(
    bytes32 indexed messageId,
    uint256 indexed attemptNumber,
    bool success,
    string result
);
```

## 👨‍💼 Subtask 3: Admin Recovery Functions

### Recovery Actions

```solidity
enum RecoveryAction {
    RETRY,          // Schedule immediate retry
    FORCE_COMPLETE, // Mark as completed (with verification)
    ROLLBACK,       // Cancel and rollback changes
    MANUAL_RESOLVE, // Mark as manually resolved
    CANCEL          // Cancel the operation
}
```

### Single Message Recovery

```javascript
// Execute recovery for single message
await errorRecoveryManager.executeRecovery(
  messageId,
  RecoveryAction.FORCE_COMPLETE,
  "Manual verification completed",
  recoveryHash // Required for high-risk actions
);
```

### Batch Recovery

```javascript
// Recover multiple messages at once
await errorRecoveryManager.executeBatchRecovery(
  [messageId1, messageId2, messageId3],
  RecoveryAction.MANUAL_RESOLVE,
  "System maintenance resolution"
);
```

### Emergency Recovery

```javascript
// Emergency recovery for critical situations
await errorRecoveryManager.emergencyRecovery(
  messageIds,
  "Critical system issue - immediate resolution required"
);
```

### Recovery Verification

For high-risk actions (FORCE_COMPLETE, ROLLBACK), a recovery hash is required:

```javascript
// Generate recovery hash
const recoveryHash = ethers.keccak256(
  ethers.concat([
    messageId,
    ethers.toBeHex(RecoveryAction.FORCE_COMPLETE),
    adminAddress,
    ethers.toBeHex(block.timestamp),
    ethers.toBeHex(block.chainid)
  ])
);
```

### Events

```solidity
event MessageRecovered(
    bytes32 indexed messageId,
    RecoveryAction indexed action,
    address indexed admin,
    string details
);

event BatchRecoveryExecuted(
    bytes32[] messageIds,
    RecoveryAction indexed action,
    address indexed admin,
    uint256 successCount
);
```

## 📊 Subtask 4: Comprehensive Event Logging and Monitoring

### Log Levels

```solidity
enum LogLevel {
    DEBUG,      // Detailed debug information
    INFO,       // General information  
    WARNING,    // Warning conditions
    ERROR,      // Error conditions
    CRITICAL    // Critical error conditions
}
```

### Structured Logging

```javascript
// Add structured log entry
await monitoringLogger.addLog(
  LogLevel.INFO,           // level
  OperationType.TRANSFER,  // operation
  messageId,               // messageId
  84532,                   // chainId
  userAddress,             // user
  "CrossChainTransfer",    // component
  "Transfer completed successfully", // message
  encodedData,             // structured data
  gasUsed                  // gasUsed
);

// Add simple log
await monitoringLogger.addSimpleLog(
  LogLevel.ERROR,
  OperationType.SYSTEM,
  "NetworkMonitor",
  "Connection to ZetaChain lost"
);
```

### Metrics Collection

#### Counter Metrics
```javascript
// Track operation counts
await monitoringLogger.updateCounter("operations_total", 1);
await monitoringLogger.updateCounter("operations_successful", 1);
```

#### Gauge Metrics
```javascript
// Track current values
await monitoringLogger.updateGauge("active_connections", 42);
await monitoringLogger.updateGauge("memory_usage_mb", 256);
```

#### Histogram Metrics
```javascript
// Track distributions
await monitoringLogger.updateHistogram("response_time_ms", 150);
await monitoringLogger.recordTiming("transfer_duration", 5000);
```

### Alert System

```javascript
// Create alert rule
const alertId = await monitoringLogger.createAlertRule(
  "High Error Rate",           // name
  "operations_failed",         // metricName
  10,                         // threshold
  3600,                       // window (1 hour)
  1800,                       // cooldown (30 min)
  "Error rate exceeded 10 failures per hour" // message
);
```

### Audit Trail

```javascript
// Record audit event
await monitoringLogger.recordAuditEvent(
  adminAddress,               // actor
  "EMERGENCY_RECOVERY",       // action
  messageId,                  // resourceId
  true,                      // success
  "Successfully recovered stuck transfer", // details
  encodedMetadata            // structured metadata
);
```

### Querying

```javascript
// Get logs for specific message
const logs = await monitoringLogger.getMessageLogs(messageId);

// Get recent logs
const recentLogs = await monitoringLogger.getRecentLogs(100);

// Get metric value
const [value, count, lastUpdated] = await monitoringLogger.getMetric("operations_total");

// Get system statistics
const [total, completed, failed, timeout, recovered, pending] = 
  await errorRecoveryManager.getSystemStats();
```

## 🔧 Operational Procedures

### Daily Monitoring

```bash
#!/bin/bash
# Daily monitoring script

# Check system health
node scripts/check-system-health.js

# Process pending retries
node scripts/process-retries.js

# Generate daily report
node scripts/generate-daily-report.js
```

### Weekly Maintenance

```bash
#!/bin/bash
# Weekly maintenance script

# Archive old logs
node scripts/archive-logs.js --days 30

# Update metrics
node scripts/update-metrics.js

# Health check report
node scripts/weekly-health-report.js
```

### Emergency Procedures

#### 1. Mass Recovery

```javascript
// Get all failed messages from last 24 hours
const now = Math.floor(Date.now() / 1000);
const yesterday = now - 86400;

const failedMessages = await getFailedMessages(yesterday, now);

// Execute batch recovery
await errorRecoveryManager.executeBatchRecovery(
  failedMessages,
  RecoveryAction.MANUAL_RESOLVE,
  "Emergency mass recovery - system incident #123"
);
```

#### 2. System Pause

```javascript
// Pause all operations
await errorRecoveryManager.pause();

// Resolve critical issues
// ... 

// Resume operations
await errorRecoveryManager.unpause();
```

## 📈 Monitoring Dashboard Integration

### Metrics Export

The system provides metrics in a format compatible with popular monitoring systems:

```javascript
// Export metrics for Prometheus/Grafana
const metrics = await getAllMetrics();
const prometheusFormat = convertToPrometheusFormat(metrics);

// Export logs for ELK stack
const logs = await getRecentLogs(1000);
const elkFormat = convertToELKFormat(logs);
```

### Sample Grafana Queries

```promql
# Error rate over time
rate(operations_failed_total[5m])

# Average response time
avg(operation_duration_ms)

# Success rate percentage
(operations_successful_total / operations_total) * 100
```

## 🛡️ Security Considerations

### Access Control

The system uses role-based access control:

- **DEFAULT_ADMIN_ROLE**: Full system administration
- **ADMIN_ROLE**: Configuration management
- **RECOVERY_ROLE**: Recovery operations
- **OPERATOR_ROLE**: Day-to-day operations
- **LOGGER_ROLE**: Logging operations
- **MONITOR_ROLE**: Metrics and monitoring
- **AUDITOR_ROLE**: Audit trail management

### Recovery Hash Verification

High-risk recovery actions require hash verification:

```javascript
// Only FORCE_COMPLETE and ROLLBACK require verification
const recoveryHash = generateRecoveryHash(messageId, action, adminAddress);
await executeRecovery(messageId, action, details, recoveryHash);
```

### Audit Requirements

All administrative actions are logged in the audit trail for compliance:

```javascript
// Every admin action generates an audit event
event AuditEventRecorded(
    uint256 indexed auditId,
    address indexed actor,
    string indexed action,
    bytes32 resourceId,
    bool success
);
```

## 🧪 Testing

### Run Error Recovery Tests

```bash
# Run comprehensive error recovery tests
npm run test:error-recovery

# Run specific test suites
npm run test -- --grep "Timeout Mechanism"
npm run test -- --grep "Retry Logic"
npm run test -- --grep "Admin Recovery"
npm run test -- --grep "Event Logging"
```

### Test Scenarios

1. **Timeout Scenarios**
   - Message timeout detection
   - Chain-specific timeout handling
   - Timeout recovery workflows

2. **Retry Scenarios**  
   - Exponential backoff calculation
   - Maximum retry enforcement
   - Automatic retry execution

3. **Recovery Scenarios**
   - Single message recovery
   - Batch recovery operations
   - Emergency recovery procedures

4. **Monitoring Scenarios**
   - Multi-level logging
   - Metrics collection
   - Alert triggering
   - Audit trail maintenance

## 📚 API Reference

### ErrorRecoveryManager

#### Core Functions
- `registerMessage(message, sender)` - Register message for monitoring
- `markMessageCompleted(messageId)` - Mark message as completed
- `reportMessageError(messageId, errorType, errorMessage, shouldRetry)` - Report processing error

#### Timeout Functions
- `handleTimeouts(messageIds)` - Process timeout detection
- `getTimedOutMessages(fromTime, toTime)` - Query timed out messages
- `updateTimeoutConfig(defaultTimeout, extendedTimeout)` - Configure timeouts
- `setChainTimeout(chainId, timeout)` - Set chain-specific timeout

#### Retry Functions
- `executeAutoRetries(maxRetries)` - Execute automatic retries
- `forceRetry(messageId, reason)` - Force retry (admin only)
- `updateRetryConfig(...)` - Configure retry behavior

#### Recovery Functions
- `executeRecovery(messageId, action, details, recoveryHash)` - Single message recovery
- `executeBatchRecovery(messageIds, action, details)` - Batch recovery
- `emergencyRecovery(messageIds, reason)` - Emergency recovery

#### Query Functions
- `getMessage(messageId)` - Get message details
- `getSystemStats()` - Get system statistics
- `checkMessageHealth(messageId)` - Check message health

### MonitoringLogger

#### Logging Functions
- `addLog(level, operation, messageId, chainId, user, component, message, data, gasUsed)` - Add structured log
- `addSimpleLog(level, operation, component, message)` - Add simple log
- `logOperation(operation, messageId, chainId, user, success, duration, errorMessage)` - Log operation

#### Metrics Functions
- `updateCounter(name, increment)` - Update counter metric
- `updateGauge(name, value)` - Update gauge metric
- `updateHistogram(name, value)` - Update histogram metric
- `recordTiming(name, duration)` - Record timing metric

#### Alert Functions
- `createAlertRule(name, metricName, threshold, windowSize, cooldown, alertMessage)` - Create alert
- `setAlertingEnabled(enabled)` - Enable/disable alerting

#### Query Functions
- `getMessageLogs(messageId)` - Get logs for message
- `getRecentLogs(count)` - Get recent logs
- `getMetric(name)` - Get metric value
- `getAllMetricNames()` - Get all metric names

## 🚀 Best Practices

### 1. Proactive Monitoring

```javascript
// Set up comprehensive monitoring
await setupDefaultAlerts();
await configureMetricsDashboard();
await enableAuditLogging();
```

### 2. Regular Health Checks

```javascript
// Daily health check routine
async function dailyHealthCheck() {
  const stats = await errorRecoveryManager.getSystemStats();
  
  if (stats.failed > stats.completed * 0.1) {
    // Alert: High failure rate
    await triggerAlert("HIGH_FAILURE_RATE", stats);
  }
  
  const pendingTimeouts = await checkPendingTimeouts();
  if (pendingTimeouts.length > 10) {
    // Alert: Many pending timeouts
    await triggerAlert("PENDING_TIMEOUTS", pendingTimeouts);
  }
}
```

### 3. Gradual Recovery

```javascript
// Recover messages gradually to avoid system overload
async function gradualRecovery(messageIds) {
  const batchSize = 5;
  
  for (let i = 0; i < messageIds.length; i += batchSize) {
    const batch = messageIds.slice(i, i + batchSize);
    await errorRecoveryManager.executeBatchRecovery(
      batch,
      RecoveryAction.RETRY,
      `Gradual recovery batch ${Math.floor(i / batchSize) + 1}`
    );
    
    // Wait between batches
    await new Promise(resolve => setTimeout(resolve, 30000));
  }
}
```

### 4. Configuration Management

```javascript
// Environment-specific configuration
const config = {
  development: {
    maxRetries: 3,
    baseDelay: 60,      // 1 minute
    alertThreshold: 5,
  },
  staging: {
    maxRetries: 5,
    baseDelay: 300,     // 5 minutes
    alertThreshold: 10,
  },
  production: {
    maxRetries: 5,
    baseDelay: 300,     // 5 minutes
    alertThreshold: 3,
  }
};
```

## 📞 Support and Troubleshooting

### Common Issues

1. **High Memory Usage**
   - Archive old logs regularly
   - Set appropriate log retention policies
   - Monitor metrics collection

2. **Retry Loops**
   - Check retry configuration
   - Verify maximum retry limits
   - Review error categorization

3. **Missing Alerts**
   - Verify alert rule configuration
   - Check cooldown periods
   - Confirm metric updates

### Getting Help

- 📖 Documentation: See `/docs` directory
- 🧪 Tests: See `/test/ErrorRecoverySystem.test.ts`
- 🔧 Configuration: See `/config` directory
- 🐛 Issues: Report on GitHub repository

---

**Universal NFT Protocol Error Recovery System** - Enterprise-grade error handling, monitoring, and recovery for cross-chain NFT operations. 🛡️⚡🔧