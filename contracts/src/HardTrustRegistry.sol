// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title HardTrustRegistry
/// @notice Minimal device registry — an authorized attester registers devices on-chain.
contract HardTrustRegistry {
    struct Device {
        address deviceAddr;
        address attester;
        uint256 attestedAt;
        bool active;
    }

    error NotAttester();
    error DeviceAlreadyRegistered(bytes32 serialHash);

    event DeviceRegistered(bytes32 indexed serialHash, address indexed deviceAddr, address indexed attester);

    address public immutable ATTESTER;

    mapping(bytes32 => Device) private devices;

    constructor(address _attester) {
        ATTESTER = _attester;
    }

    /// @notice Register a device. Only the authorized attester may call this.
    /// @param serialHash keccak256 of the device hardware serial number
    /// @param deviceAddr Ethereum address derived from the device's public key
    function registerDevice(bytes32 serialHash, address deviceAddr) external {
        if (msg.sender != ATTESTER) revert NotAttester();
        if (devices[serialHash].active) revert DeviceAlreadyRegistered(serialHash);
        devices[serialHash] =
            Device({deviceAddr: deviceAddr, attester: msg.sender, attestedAt: block.timestamp, active: true});
        emit DeviceRegistered(serialHash, deviceAddr, msg.sender);
    }

    /// @notice Query a device record by serial hash. Returns zero values if not found.
    function getDevice(bytes32 serialHash)
        external
        view
        returns (address deviceAddr, address attester_, uint256 attestedAt, bool active)
    {
        Device storage d = devices[serialHash];
        return (d.deviceAddr, d.attester, d.attestedAt, d.active);
    }

    /// @notice Check whether an address is the authorized attester.
    function isAttester(address addr) external view returns (bool) {
        return addr == ATTESTER;
    }
}
