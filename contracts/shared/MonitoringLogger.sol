// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "./ErrorHandling.sol";
import "../../shared/CrossChainMessage.sol";

/**
 * @title MonitoringLogger
 * @notice Comprehensive monitoring and logging system for cross-chain NFT operations
 * @dev Provides structured logging, metrics collection, and audit trail functionality
 */
contract MonitoringLogger is Ownable, AccessControl, ReentrancyGuard {
    using ErrorHandling for ErrorHandling.PendingMessage;

    // ============ Access Control Roles ============
    
    bytes32 public constant LOGGER_ROLE = keccak256("LOGGER_ROLE");
    bytes32 public constant MONITOR_ROLE = keccak256("MONITOR_ROLE");
    bytes32 public constant AUDITOR_ROLE = keccak256("AUDITOR_ROLE");

    // ============ Enums ============

    enum LogLevel {
        DEBUG,      // Detailed debug information
        INFO,       // General information
        WARNING,    // Warning conditions
        ERROR,      // Error conditions
        CRITICAL    // Critical error conditions
    }

    enum OperationType {
        MINT,           // NFT minting
        BURN,           // NFT burning
        TRANSFER,       // Cross-chain transfer
        RECEIVE,        // Receiving cross-chain NFT
        RETRY,          // Retry operation
        RECOVERY,       // Admin recovery
        TIMEOUT,        // Message timeout
        REVERT,         // Transaction revert
        SYSTEM          // System operations
    }

    enum MetricType {
        COUNTER,        // Incrementing counter
        GAUGE,          // Current value
        HISTOGRAM,      // Distribution of values
        TIMING          // Timing measurements
    }

    // ============ Structs ============

    struct LogEntry {
        uint256 id;
        uint256 timestamp;
        LogLevel level;
        OperationType operation;
        bytes32 messageId;
        uint32 chainId;
        address user;
        string component;
        string message;
        bytes data;
        uint256 gasUsed;
        bool isStructured;
    }

    struct Metric {
        string name;
        MetricType metricType;
        uint256 value;
        uint256 count;
        uint256 sum;
        uint256 min;
        uint256 max;
        uint256 lastUpdated;
        mapping(string => uint256) labels; // For dimensional metrics
    }

    struct AlertRule {
        uint256 id;
        string name;
        string metricName;
        uint256 threshold;
        uint256 windowSize;
        bool isActive;
        uint256 lastTriggered;
        uint256 cooldown;
        string alertMessage;
    }

    struct AuditEvent {
        uint256 id;
        uint256 timestamp;
        address actor;
        string action;
        bytes32 resourceId;
        bool success;
        string details;
        bytes metadata;
    }

    // ============ State Variables ============

    // Logging
    uint256 private _logIdCounter;
    mapping(uint256 => LogEntry) public logs;
    mapping(bytes32 => uint256[]) public messageToLogs; // messageId => logIds[]
    mapping(address => uint256[]) public userToLogs; // user => logIds[]
    uint256[] public allLogIds;
    
    // Metrics
    mapping(string => Metric) public metrics;
    string[] public metricNames;
    
    // Alerts
    uint256 private _alertIdCounter;
    mapping(uint256 => AlertRule) public alertRules;
    uint256[] public activeAlerts;
    
    // Audit trail
    uint256 private _auditIdCounter;
    mapping(uint256 => AuditEvent) public auditTrail;
    uint256[] public allAuditIds;
    
    // Configuration
    LogLevel public minLogLevel = LogLevel.INFO;
    uint256 public maxLogRetention = 30 days;
    uint256 public maxLogsPerBatch = 100;
    bool public loggingEnabled = true;
    bool public metricsEnabled = true;
    bool public alertingEnabled = true;

    // ============ Events ============

    event LogEntryAdded(
        uint256 indexed logId,
        LogLevel indexed level,
        OperationType indexed operation,
        bytes32 messageId,
        string component,
        string message
    );

    event MetricUpdated(
        string indexed metricName,
        MetricType indexed metricType,
        uint256 value,
        uint256 timestamp
    );

    event AlertTriggered(
        uint256 indexed alertId,
        string indexed metricName,
        uint256 currentValue,
        uint256 threshold,
        string alertMessage
    );

    event AuditEventRecorded(
        uint256 indexed auditId,
        address indexed actor,
        string indexed action,
        bytes32 resourceId,
        bool success
    );

    event ConfigurationChanged(
        string indexed parameter,
        uint256 oldValue,
        uint256 newValue,
        address indexed admin
    );

    event LogsArchived(
        uint256 fromLogId,
        uint256 toLogId,
        uint256 archivedCount,
        string archiveLocation
    );

    // ============ Constructor ============

    constructor(address initialOwner) Ownable(initialOwner) {
        _grantRole(DEFAULT_ADMIN_ROLE, initialOwner);
        _grantRole(LOGGER_ROLE, initialOwner);
        _grantRole(MONITOR_ROLE, initialOwner);
        _grantRole(AUDITOR_ROLE, initialOwner);

        // Initialize default metrics
        _initializeDefaultMetrics();
    }

    // ============ Core Logging Functions ============

    /**
     * @notice Add a structured log entry
     * @param level Log level
     * @param operation Operation type
     * @param messageId Associated message ID (if applicable)
     * @param chainId Chain ID where operation occurred
     * @param user User involved in operation
     * @param component System component generating log
     * @param message Human-readable log message
     * @param data Additional structured data
     * @param gasUsed Gas consumed by operation
     */
    function addLog(
        LogLevel level,
        OperationType operation,
        bytes32 messageId,
        uint32 chainId,
        address user,
        string calldata component,
        string calldata message,
        bytes calldata data,
        uint256 gasUsed
    ) external onlyRole(LOGGER_ROLE) returns (uint256 logId) {
        return _addLog(level, operation, messageId, chainId, user, component, message, data, gasUsed);
    }

    function _addLog(
        LogLevel level,
        OperationType operation,
        bytes32 messageId,
        uint32 chainId,
        address user,
        string memory component,
        string memory message,
        bytes memory data,
        uint256 gasUsed
    ) internal returns (uint256 logId) {
        require(loggingEnabled, "Logging disabled");
        require(level >= minLogLevel, "Log level too low");

        logId = ++_logIdCounter;
        
        LogEntry storage logEntry = logs[logId];
        logEntry.id = logId;
        logEntry.timestamp = block.timestamp;
        logEntry.level = level;
        logEntry.operation = operation;
        logEntry.messageId = messageId;
        logEntry.chainId = chainId;
        logEntry.user = user;
        logEntry.component = component;
        logEntry.message = message;
        logEntry.data = data;
        logEntry.gasUsed = gasUsed;
        logEntry.isStructured = data.length > 0;

        // Index the log
        allLogIds.push(logId);
        if (messageId != bytes32(0)) {
            messageToLogs[messageId].push(logId);
        }
        if (user != address(0)) {
            userToLogs[user].push(logId);
        }

        emit LogEntryAdded(logId, level, operation, messageId, component, message);

        // Update metrics
        _updateLogMetrics(level, operation);

        return logId;
    }

    /**
     * @notice Add a simple log entry
     * @param level Log level
     * @param operation Operation type
     * @param component System component
     * @param message Log message
     */
    function addSimpleLog(
        LogLevel level,
        OperationType operation,
        string calldata component,
        string calldata message
    ) external onlyRole(LOGGER_ROLE) returns (uint256 logId) {
        return _addLog(
            level,
            operation,
            bytes32(0),
            0,
            address(0),
            component,
            message,
            "",
            0
        );
    }

    /**
     * @notice Add operation-specific log with automatic metric updates
     * @param operation Operation type
     * @param messageId Message ID
     * @param chainId Chain ID
     * @param user User address
     * @param success Whether operation was successful
     * @param duration Operation duration in milliseconds
     * @param errorMessage Error message (if failed)
     */
    function logOperation(
        OperationType operation,
        bytes32 messageId,
        uint32 chainId,
        address user,
        bool success,
        uint256 duration,
        string calldata errorMessage
    ) external onlyRole(LOGGER_ROLE) {
        // Log the operation
        LogLevel level = success ? LogLevel.INFO : LogLevel.ERROR;
        string memory message = success 
            ? _getSuccessMessage(operation)
            : string(abi.encodePacked(_getFailureMessage(operation), ": ", errorMessage));

        bytes memory data = abi.encode(success, duration, errorMessage);

        _addLog(
            level,
            operation,
            messageId,
            chainId,
            user,
            "Operation",
            message,
            data,
            0
        );

        // Update operation metrics
        _updateOperationMetrics(operation, success, duration);
    }

    // ============ Metrics Functions ============

    /**
     * @notice Update a counter metric
     * @param name Metric name
     * @param increment Value to add to counter
     */
    function updateCounter(
        string calldata name,
        uint256 increment
    ) external onlyRole(MONITOR_ROLE) {
        require(metricsEnabled, "Metrics disabled");

        Metric storage metric = metrics[name];
        if (metric.lastUpdated == 0) {
            // First time - initialize
            metric.name = name;
            metric.metricType = MetricType.COUNTER;
            metricNames.push(name);
        }
        
        require(metric.metricType == MetricType.COUNTER, "Metric type mismatch");
        
        metric.value += increment;
        metric.count++;
        metric.lastUpdated = block.timestamp;

        emit MetricUpdated(name, MetricType.COUNTER, metric.value, block.timestamp);
        
        _checkAlerts(name, metric.value);
    }

    /**
     * @notice Update a gauge metric (current value)
     * @param name Metric name
     * @param value Current value
     */
    function updateGauge(
        string calldata name,
        uint256 value
    ) external onlyRole(MONITOR_ROLE) {
        require(metricsEnabled, "Metrics disabled");

        Metric storage metric = metrics[name];
        if (metric.lastUpdated == 0) {
            metric.name = name;
            metric.metricType = MetricType.GAUGE;
            metricNames.push(name);
        }
        
        require(metric.metricType == MetricType.GAUGE, "Metric type mismatch");
        
        metric.value = value;
        metric.lastUpdated = block.timestamp;

        emit MetricUpdated(name, MetricType.GAUGE, value, block.timestamp);
        
        _checkAlerts(name, value);
    }

    /**
     * @notice Update a histogram metric (for distributions)
     * @param name Metric name
     * @param value New sample value
     */
    function updateHistogram(
        string calldata name,
        uint256 value
    ) external onlyRole(MONITOR_ROLE) {
        _updateHistogram(name, value);
    }

    function _updateHistogram(
        string memory name,
        uint256 value
    ) internal {
        require(metricsEnabled, "Metrics disabled");

        Metric storage metric = metrics[name];
        if (metric.lastUpdated == 0) {
            metric.name = name;
            metric.metricType = MetricType.HISTOGRAM;
            metric.min = type(uint256).max;
            metricNames.push(name);
        }
        
        require(metric.metricType == MetricType.HISTOGRAM, "Metric type mismatch");
        
        metric.count++;
        metric.sum += value;
        metric.value = metric.sum / metric.count; // Average
        
        if (value < metric.min) {
            metric.min = value;
        }
        if (value > metric.max) {
            metric.max = value;
        }
        
        metric.lastUpdated = block.timestamp;

        emit MetricUpdated(name, MetricType.HISTOGRAM, metric.value, block.timestamp);
        
        _checkAlerts(name, metric.value);
    }

    /**
     * @notice Record timing metric
     * @param name Metric name
     * @param duration Duration in milliseconds
     */
    function recordTiming(
        string calldata name,
        uint256 duration
    ) external onlyRole(MONITOR_ROLE) {
        _updateHistogram(string(abi.encodePacked(name, "_duration_ms")), duration);
    }

    // ============ Alerting Functions ============

    /**
     * @notice Create an alert rule
     * @param name Alert rule name
     * @param metricName Metric to monitor
     * @param threshold Alert threshold
     * @param windowSize Time window for evaluation
     * @param cooldown Minimum time between alerts
     * @param alertMessage Alert message template
     */
    function createAlertRule(
        string calldata name,
        string calldata metricName,
        uint256 threshold,
        uint256 windowSize,
        uint256 cooldown,
        string calldata alertMessage
    ) external onlyRole(MONITOR_ROLE) returns (uint256 alertId) {
        alertId = ++_alertIdCounter;
        
        AlertRule storage rule = alertRules[alertId];
        rule.id = alertId;
        rule.name = name;
        rule.metricName = metricName;
        rule.threshold = threshold;
        rule.windowSize = windowSize;
        rule.isActive = true;
        rule.cooldown = cooldown;
        rule.alertMessage = alertMessage;
        
        activeAlerts.push(alertId);
        
        return alertId;
    }

    /**
     * @notice Check and trigger alerts for a metric
     * @param metricName Metric name to check
     * @param currentValue Current metric value
     */
    function _checkAlerts(string memory metricName, uint256 currentValue) internal {
        if (!alertingEnabled) return;

        for (uint256 i = 0; i < activeAlerts.length; i++) {
            uint256 alertId = activeAlerts[i];
            AlertRule storage rule = alertRules[alertId];
            
            if (!rule.isActive) continue;
            if (keccak256(bytes(rule.metricName)) != keccak256(bytes(metricName))) continue;
            if (block.timestamp < rule.lastTriggered + rule.cooldown) continue;
            
            if (currentValue >= rule.threshold) {
                rule.lastTriggered = block.timestamp;
                
                emit AlertTriggered(
                    alertId,
                    metricName,
                    currentValue,
                    rule.threshold,
                    rule.alertMessage
                );
                
                // Log the alert
                _addLog(
                    LogLevel.WARNING,
                    OperationType.SYSTEM,
                    bytes32(0),
                    0,
                    address(0),
                    "Alerting",
                    string(abi.encodePacked("Alert triggered: ", rule.alertMessage)),
                    "",
                    0
                );
            }
        }
    }

    // ============ Audit Trail Functions ============

    /**
     * @notice Record an audit event
     * @param actor Address performing the action
     * @param action Description of action
     * @param resourceId Resource being acted upon
     * @param success Whether action was successful
     * @param details Additional details
     * @param metadata Structured metadata
     */
    function recordAuditEvent(
        address actor,
        string calldata action,
        bytes32 resourceId,
        bool success,
        string calldata details,
        bytes calldata metadata
    ) external onlyRole(AUDITOR_ROLE) returns (uint256 auditId) {
        auditId = ++_auditIdCounter;
        
        AuditEvent storage auditEvent = auditTrail[auditId];
        auditEvent.id = auditId;
        auditEvent.timestamp = block.timestamp;
        auditEvent.actor = actor;
        auditEvent.action = action;
        auditEvent.resourceId = resourceId;
        auditEvent.success = success;
        auditEvent.details = details;
        auditEvent.metadata = metadata;
        
        allAuditIds.push(auditId);
        
        emit AuditEventRecorded(auditId, actor, action, resourceId, success);
        
        return auditId;
    }

    // ============ Query Functions ============

    /**
     * @notice Get logs for a specific message
     * @param messageId Message ID to query
     * @return logIds Array of log IDs for the message
     */
    function getMessageLogs(bytes32 messageId) external view returns (uint256[] memory logIds) {
        return messageToLogs[messageId];
    }

    /**
     * @notice Get logs for a specific user
     * @param user User address to query
     * @return logIds Array of log IDs for the user
     */
    function getUserLogs(address user) external view returns (uint256[] memory logIds) {
        return userToLogs[user];
    }

    /**
     * @notice Get recent logs
     * @param count Maximum number of logs to return
     * @return logIds Array of recent log IDs
     */
    function getRecentLogs(uint256 count) external view returns (uint256[] memory logIds) {
        uint256 totalLogs = allLogIds.length;
        if (totalLogs == 0) {
            return new uint256[](0);
        }
        
        uint256 returnCount = count > totalLogs ? totalLogs : count;
        logIds = new uint256[](returnCount);
        
        for (uint256 i = 0; i < returnCount; i++) {
            logIds[i] = allLogIds[totalLogs - 1 - i];
        }
        
        return logIds;
    }

    /**
     * @notice Get metric value
     * @param name Metric name
     * @return value Current metric value
     * @return count Number of updates (for histograms)
     * @return lastUpdated Timestamp of last update
     */
    function getMetric(string calldata name) external view returns (
        uint256 value,
        uint256 count,
        uint256 lastUpdated
    ) {
        Metric storage metric = metrics[name];
        return (metric.value, metric.count, metric.lastUpdated);
    }

    /**
     * @notice Get all metric names
     * @return names Array of metric names
     */
    function getAllMetricNames() external view returns (string[] memory names) {
        return metricNames;
    }

    // ============ Internal Helper Functions ============

    /**
     * @notice Initialize default metrics
     */
    function _initializeDefaultMetrics() internal {
        // Operation counters
        metrics["operations_total"].metricType = MetricType.COUNTER;
        metrics["operations_successful"].metricType = MetricType.COUNTER;
        metrics["operations_failed"].metricType = MetricType.COUNTER;
        
        // Error counters by level
        metrics["logs_debug"].metricType = MetricType.COUNTER;
        metrics["logs_info"].metricType = MetricType.COUNTER;
        metrics["logs_warning"].metricType = MetricType.COUNTER;
        metrics["logs_error"].metricType = MetricType.COUNTER;
        metrics["logs_critical"].metricType = MetricType.COUNTER;
        
        // Operation timings
        metrics["operation_duration_ms"].metricType = MetricType.HISTOGRAM;
        metrics["operation_duration_ms"].min = type(uint256).max;

        // Add metric names
        metricNames.push("operations_total");
        metricNames.push("operations_successful");
        metricNames.push("operations_failed");
        metricNames.push("logs_debug");
        metricNames.push("logs_info");
        metricNames.push("logs_warning");
        metricNames.push("logs_error");
        metricNames.push("logs_critical");
        metricNames.push("operation_duration_ms");
    }

    /**
     * @notice Update metrics based on log entry
     */
    function _updateLogMetrics(LogLevel level, OperationType operation) internal {
        // Update log level counters
        if (level == LogLevel.DEBUG) {
            metrics["logs_debug"].value++;
        } else if (level == LogLevel.INFO) {
            metrics["logs_info"].value++;
        } else if (level == LogLevel.WARNING) {
            metrics["logs_warning"].value++;
        } else if (level == LogLevel.ERROR) {
            metrics["logs_error"].value++;
        } else if (level == LogLevel.CRITICAL) {
            metrics["logs_critical"].value++;
        }
    }

    /**
     * @notice Update operation-specific metrics
     */
    function _updateOperationMetrics(
        OperationType operation,
        bool success,
        uint256 duration
    ) internal {
        metrics["operations_total"].value++;
        
        if (success) {
            metrics["operations_successful"].value++;
        } else {
            metrics["operations_failed"].value++;
        }
        
        if (duration > 0) {
            _updateHistogram("operation_duration_ms", duration);
        }
    }

    /**
     * @notice Get success message for operation type
     */
    function _getSuccessMessage(OperationType operation) internal pure returns (string memory) {
        if (operation == OperationType.MINT) return "NFT minted successfully";
        if (operation == OperationType.BURN) return "NFT burned successfully";
        if (operation == OperationType.TRANSFER) return "Cross-chain transfer completed";
        if (operation == OperationType.RECEIVE) return "Cross-chain NFT received";
        if (operation == OperationType.RETRY) return "Operation retry successful";
        if (operation == OperationType.RECOVERY) return "Admin recovery completed";
        return "Operation completed successfully";
    }

    /**
     * @notice Get failure message for operation type
     */
    function _getFailureMessage(OperationType operation) internal pure returns (string memory) {
        if (operation == OperationType.MINT) return "NFT mint failed";
        if (operation == OperationType.BURN) return "NFT burn failed";
        if (operation == OperationType.TRANSFER) return "Cross-chain transfer failed";
        if (operation == OperationType.RECEIVE) return "Cross-chain receive failed";
        if (operation == OperationType.RETRY) return "Operation retry failed";
        if (operation == OperationType.RECOVERY) return "Admin recovery failed";
        return "Operation failed";
    }

    // ============ Admin Functions ============

    /**
     * @notice Set minimum log level
     * @param level New minimum log level
     */
    function setMinLogLevel(LogLevel level) external onlyOwner {
        LogLevel oldLevel = minLogLevel;
        minLogLevel = level;
        emit ConfigurationChanged("MIN_LOG_LEVEL", uint256(oldLevel), uint256(level), msg.sender);
    }

    /**
     * @notice Enable/disable logging
     * @param enabled Whether logging should be enabled
     */
    function setLoggingEnabled(bool enabled) external onlyOwner {
        loggingEnabled = enabled;
        emit ConfigurationChanged("LOGGING_ENABLED", loggingEnabled ? 1 : 0, enabled ? 1 : 0, msg.sender);
    }

    /**
     * @notice Enable/disable metrics
     * @param enabled Whether metrics should be enabled
     */
    function setMetricsEnabled(bool enabled) external onlyOwner {
        metricsEnabled = enabled;
        emit ConfigurationChanged("METRICS_ENABLED", metricsEnabled ? 1 : 0, enabled ? 1 : 0, msg.sender);
    }

    /**
     * @notice Enable/disable alerting
     * @param enabled Whether alerting should be enabled
     */
    function setAlertingEnabled(bool enabled) external onlyOwner {
        alertingEnabled = enabled;
        emit ConfigurationChanged("ALERTING_ENABLED", alertingEnabled ? 1 : 0, enabled ? 1 : 0, msg.sender);
    }

    /**
     * @notice Archive old logs
     * @param beforeTimestamp Archive logs before this timestamp
     * @param archiveLocation Description of where logs are archived
     */
    function archiveLogs(
        uint256 beforeTimestamp,
        string calldata archiveLocation
    ) external onlyOwner returns (uint256 archivedCount) {
        uint256 minLogId = type(uint256).max;
        uint256 maxLogId = 0;
        archivedCount = 0;

        for (uint256 i = 0; i < allLogIds.length; i++) {
            uint256 logId = allLogIds[i];
            if (logs[logId].timestamp < beforeTimestamp) {
                if (logId < minLogId) minLogId = logId;
                if (logId > maxLogId) maxLogId = logId;
                delete logs[logId];
                archivedCount++;
            }
        }

        if (archivedCount > 0) {
            emit LogsArchived(minLogId, maxLogId, archivedCount, archiveLocation);
        }

        return archivedCount;
    }

    /**
     * @notice Emergency clear all data
     */
    function emergencyClearAllData() external onlyOwner {
        // Clear all mappings and arrays
        for (uint256 i = 0; i < allLogIds.length; i++) {
            delete logs[allLogIds[i]];
        }
        delete allLogIds;
        
        for (uint256 i = 0; i < metricNames.length; i++) {
            delete metrics[metricNames[i]];
        }
        delete metricNames;
        
        for (uint256 i = 0; i < activeAlerts.length; i++) {
            delete alertRules[activeAlerts[i]];
        }
        delete activeAlerts;
        
        for (uint256 i = 0; i < allAuditIds.length; i++) {
            delete auditTrail[allAuditIds[i]];
        }
        delete allAuditIds;

        // Reinitialize
        _initializeDefaultMetrics();
    }
}