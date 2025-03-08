// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use disks::BlockDevice;
use log::{debug, info, trace, warn};
use partitioning::{
    planner::{Planner, PARTITION_ALIGNMENT},
    strategy::{AllocationStrategy, PartitionRequest, SizeRequirement, Strategy},
};

use crate::{commands::Command, Constraints, StrategyDefinition};

/// Provisioner
pub struct Provisioner {
    /// Pool of devices
    devices: Vec<BlockDevice>,

    /// Strategy configurations
    configs: HashMap<String, StrategyDefinition>,
}

/// Compiled plan
pub struct Plan<'a> {
    pub strategy: &'a StrategyDefinition,
    pub device_assignments: HashMap<String, DevicePlan<'a>>,
}

#[derive(Debug, Clone)]
pub struct DevicePlan<'a> {
    pub device: &'a BlockDevice,
    pub planner: Planner,
    pub strategy: Strategy,
}

impl Default for Provisioner {
    fn default() -> Self {
        Self::new()
    }
}

impl Provisioner {
    /// Create a new provisioner
    pub fn new() -> Self {
        debug!("Creating new provisioner");
        Self {
            devices: Vec::new(),
            configs: HashMap::new(),
        }
    }

    /// Add a strategy configuration
    pub fn add_strategy(&mut self, config: StrategyDefinition) {
        info!("Adding strategy: {}", config.name);
        self.configs.insert(config.name.clone(), config);
    }

    // Add a device to the provisioner pool
    pub fn push_device(&mut self, device: BlockDevice) {
        debug!("Adding device to pool: {:?}", device);
        self.devices.push(device)
    }

    // Build an inheritance chain for a strategy
    fn strategy_parents<'a>(&'a self, strategy: &'a StrategyDefinition) -> Vec<&'a StrategyDefinition> {
        trace!("Building inheritance chain for strategy: {}", strategy.name);
        let mut chain = vec![];
        if let Some(parent) = &strategy.inherits {
            if let Some(parent) = self.configs.get(parent) {
                chain.extend(self.strategy_parents(parent));
            }
        }
        chain.push(strategy);
        chain
    }

    /// Attempt all strategies on the pool of devices
    pub fn plan(&self) -> Vec<Plan> {
        info!("Planning device provisioning");
        let mut plans = Vec::new();
        for strategy in self.configs.values() {
            debug!("Attempting strategy: {}", strategy.name);
            self.create_plans_for_strategy(strategy, &mut HashMap::new(), &mut plans);
        }
        debug!("Generated {} plans", plans.len());
        plans
    }

    fn create_plans_for_strategy<'a>(
        &'a self,
        strategy: &'a StrategyDefinition,
        device_assignments: &mut HashMap<String, DevicePlan<'a>>,
        plans: &mut Vec<Plan<'a>>,
    ) {
        trace!("Creating plans for strategy: {}", strategy.name);
        let chain = self.strategy_parents(strategy);

        for command in chain.iter().flat_map(|s| &s.commands) {
            match command {
                Command::FindDisk(command) => {
                    // Skip if already assigned
                    if device_assignments.contains_key(&command.name) {
                        trace!("Disk {} already assigned, skipping", command.name);
                        continue;
                    }

                    // Find matching devices that haven't been assigned yet
                    let matching_devices: Vec<_> = self
                        .devices
                        .iter()
                        .filter(|d| match command.constraints.as_ref() {
                            Some(Constraints::AtLeast(n)) => d.size() >= *n,
                            Some(Constraints::Exact(n)) => d.size() == *n,
                            Some(Constraints::Range { min, max }) => d.size() >= *min && d.size() <= *max,
                            _ => true,
                        })
                        .filter(|d| {
                            !device_assignments
                                .values()
                                .any(|assigned| std::ptr::eq(assigned.device, *d))
                        })
                        .collect();

                    debug!("Found {} matching devices for {}", matching_devices.len(), command.name);

                    // Branch for each matching device
                    for device in matching_devices {
                        trace!("Creating plan branch for device: {:?}", device);
                        let mut new_assignments = device_assignments.clone();
                        new_assignments.insert(
                            command.name.clone(),
                            DevicePlan {
                                device,
                                planner: Planner::new(device)
                                    .with_start_offset(PARTITION_ALIGNMENT)
                                    .with_end_offset(device.size() - PARTITION_ALIGNMENT),
                                strategy: Strategy::new(AllocationStrategy::LargestFree),
                            },
                        );
                        self.create_plans_for_strategy(strategy, &mut new_assignments, plans);
                    }

                    return;
                }
                Command::CreatePartitionTable(command) => {
                    if let Some(device_plan) = device_assignments.get_mut(&command.disk) {
                        debug!("Creating partition table on disk {}", command.disk);
                        device_plan.strategy = Strategy::new(AllocationStrategy::InitializeWholeDisk);
                    } else {
                        warn!("Could not find disk {} to create partition table", command.disk);
                    }
                }
                Command::CreatePartition(command) => {
                    if let Some(device_plan) = device_assignments.get_mut(&command.disk) {
                        debug!("Adding partition request for disk {}", command.disk);
                        device_plan.strategy.add_request(PartitionRequest {
                            size: match &command.constraints {
                                Constraints::AtLeast(n) => SizeRequirement::AtLeast(*n),
                                Constraints::Exact(n) => SizeRequirement::Exact(*n),
                                Constraints::Range { min, max } => SizeRequirement::Range { min: *min, max: *max },
                                _ => SizeRequirement::Remaining,
                            },
                            attributes: Some(command.attributes()),
                        });
                    } else {
                        warn!("Could not find disk {} to create partition", command.disk);
                    }
                }
            }
        }

        // OK lets now apply any mutations to the device assignments
        for (disk_name, device_plan) in device_assignments.iter_mut() {
            debug!("Applying device plan for disk {}", disk_name);
            if let Err(e) = device_plan.strategy.apply(&mut device_plan.planner) {
                warn!("Failed to apply strategy for disk {}: {:?}", disk_name, e);
            }
        }

        // All commands processed successfully - create a plan
        debug!("Creating final plan for strategy {}", strategy.name);
        plans.push(Plan {
            strategy,
            device_assignments: device_assignments.clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use disks::mock::MockDisk;
    use test_log::test;

    use crate::Parser;

    use super::*;

    #[test]
    fn test_use_whole_disk() {
        let test_strategies = Parser::new_for_path("tests/use_whole_disk.kdl").unwrap();
        let def = test_strategies.strategies;
        let device = BlockDevice::mock_device(MockDisk::new(150 * 1024 * 1024 * 1024));
        let mut provisioner = Provisioner::new();
        provisioner.push_device(device);
        for def in def {
            provisioner.add_strategy(def);
        }

        let plans = provisioner.plan();
        assert_eq!(plans.len(), 1);

        let plan = &plans[0];
        assert_eq!(plan.device_assignments.len(), 1);

        for plan in plans {
            eprintln!("Plan: {}", plan.strategy.name);
            for (disk, device_plan) in plan.device_assignments.iter() {
                println!("strategy for {disk} is now: {}", device_plan.strategy.describe());
                println!("After: {}", device_plan.planner.describe_changes());
            }
        }
    }
}
