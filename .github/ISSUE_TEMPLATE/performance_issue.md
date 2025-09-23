---
name: Performance Issue
about: Report a performance problem or regression
title: '[PERFORMANCE] '
labels: ['performance', 'needs-triage']
assignees: ''
---

## Performance Issue Summary
Brief description of the performance problem.

## Environment
- **OS**: [e.g. Ubuntu 22.04]
- **Hardware**: [e.g. 8GB RAM, 4-core CPU, SSD]
- **Rust Version**: [e.g. 1.70.0] (run `rustc --version`)
- **Build Type**: [debug/release]
- **Campfire Version**: [e.g. v1.0.0 or commit hash]

## Reproduction Steps
1. Step 1
2. Step 2
3. Step 3

## Performance Metrics
- **Current Performance**: [e.g. 500ms response time, 80% CPU usage]
- **Expected Performance**: [e.g. <100ms response time, <50% CPU usage]
- **Resource Usage**: [e.g. Memory: 2GB, CPU: 80%, Disk I/O: high]

## Measurement Details
### Response Times
- Endpoint/Operation: [time measurement]
- Endpoint/Operation: [time measurement]

### Resource Usage
- Memory consumption: [measurement]
- CPU utilization: [measurement]
- Disk I/O: [measurement]
- Network I/O: [measurement]

## Profiling Data
If you have profiling data, please attach it or provide relevant excerpts:

```
Paste profiling output here (flamegraph, perf, etc.)
```

## Regression Information
- [ ] This is a new performance issue
- [ ] This is a performance regression
- [ ] Performance was acceptable in version: [version]
- [ ] Performance degraded starting in version: [version]

## Load Conditions
- **Concurrent Users**: [number]
- **Request Rate**: [requests per second]
- **Data Size**: [amount of data being processed]
- **Duration**: [how long the performance issue persists]

## Impact Assessment
- [ ] Low - Barely noticeable
- [ ] Medium - Noticeable but not blocking
- [ ] High - Significantly impacts user experience
- [ ] Critical - Makes the application unusable

## Additional Context
Any other relevant information about the performance issue.

## Potential Solutions
If you have ideas for performance improvements, please share them.

## Checklist
- [ ] I have measured the performance issue with specific metrics
- [ ] I have provided environment and hardware details
- [ ] I have described the expected vs actual performance
- [ ] I have included profiling data if available