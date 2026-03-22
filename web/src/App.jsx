import React, { Component, useEffect, useMemo, useState } from "react";
import {
  BrowserProvider,
  Contract,
  JsonRpcProvider,
  formatEther,
  getBytes,
  isAddress,
  keccak256,
  toUtf8Bytes,
} from "ethers";
import { appConfig, registryAbi } from "./contract";

const emptyForm = {
  serial: "",
  deviceAddress: "",
};

class AppErrorBoundary extends Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false, errorMessage: "" };
  }

  static getDerivedStateFromError(error) {
    return {
      hasError: true,
      errorMessage: error?.message || "Unknown frontend error",
    };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="shell">
          <section className="panel">
            <div className="section-head">
              <div>
                <p className="eyebrow">Frontend error</p>
                <h2>The page hit a runtime error before rendering fully.</h2>
              </div>
            </div>
            <p className="message error">{this.state.errorMessage}</p>
            <p className="lead">
              Refresh the page after the frontend reloads. If the error stays, inspect the browser
              console and the wallet extension injected into the page.
            </p>
          </section>
        </div>
      );
    }

    return this.props.children;
  }
}

function shortAddress(value) {
  if (!value) return "Not connected";
  return `${value.slice(0, 6)}...${value.slice(-4)}`;
}

function formatTimestamp(value) {
  if (!value) return "Pending";
  return new Intl.DateTimeFormat("en-US", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(Number(value) * 1000));
}

function extractError(error) {
  return (
    error?.shortMessage ||
    error?.reason ||
    error?.info?.error?.message ||
    error?.message ||
    "Unknown contract error"
  );
}

function normalizeRegisteredDevice(serialHash, details) {
  return {
    serialHash,
    deviceAddr: details.deviceAddr,
    attester: details.attester_ || details.attester,
    attestedAt: Number(details.attestedAt),
    active: details.active,
  };
}

function bytesToHex(bytes) {
  return Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
}

async function hashFile(file) {
  const buffer = await file.arrayBuffer();
  const hashBuffer = await crypto.subtle.digest("SHA-256", buffer);

  return {
    hash: `sha256:${bytesToHex(new Uint8Array(hashBuffer))}`,
    size: buffer.byteLength,
  };
}

async function parseJsonFile(file) {
  return JSON.parse(await file.text());
}

function computeContentHash(files) {
  const encoder = new TextEncoder();
  const sortedFiles = [...files].sort((left, right) => left.name.localeCompare(right.name));
  const parts = sortedFiles.flatMap((file) => [
    ...encoder.encode(file.name),
    ...encoder.encode(file.hash.replace("sha256:", "")),
  ]);

  return crypto.subtle
    .digest("SHA-256", new Uint8Array(parts))
    .then((hashBuffer) => `sha256:${bytesToHex(new Uint8Array(hashBuffer))}`);
}

function concatBytes(...parts) {
  const totalLength = parts.reduce((sum, part) => sum + part.length, 0);
  const result = new Uint8Array(totalLength);
  let offset = 0;

  parts.forEach((part) => {
    result.set(part, offset);
    offset += part.length;
  });

  return result;
}

function bigintToUint64Bytes(value) {
  const result = new Uint8Array(8);
  let current = value;

  for (let index = 7; index >= 0; index -= 1) {
    result[index] = Number(current & 0xffn);
    current >>= 8n;
  }

  return result;
}

function sha256StringToBytes32(value) {
  if (typeof value !== "string") {
    throw new Error("Expected sha256 string.");
  }

  const hex = value.startsWith("sha256:") ? value.slice(7) : value;
  if (!/^[0-9a-fA-F]{64}$/.test(hex)) {
    throw new Error(`Invalid SHA-256 hash: ${value}`);
  }

  return `0x${hex.toLowerCase()}`;
}

function parseCaptureSignature(signatureHex) {
  const normalized = signatureHex?.trim().replace(/^0x/i, "") || "";
  if (!/^[0-9a-fA-F]{130}$/.test(normalized)) {
    throw new Error("Invalid capture signature.");
  }

  const recoveryId = Number.parseInt(normalized.slice(128, 130), 16);

  return {
    r: `0x${normalized.slice(0, 64)}`,
    s: `0x${normalized.slice(64, 128)}`,
    v: recoveryId + 27,
  };
}

function computeCapturePrehash(capture) {
  const contentHash = sha256StringToBytes32(capture.content_hash);
  const timestampSeconds = Date.parse(capture.timestamp);

  if (Number.isNaN(timestampSeconds)) {
    throw new Error("Invalid capture timestamp.");
  }

  const encoder = new TextEncoder();
  const preimage = concatBytes(
    getBytes(keccak256(toUtf8Bytes(capture.serial))),
    getBytes(capture.address),
    getBytes(contentHash),
    bigintToUint64Bytes(BigInt(Math.floor(timestampSeconds / 1000))),
    encoder.encode(capture.environment.script_hash),
    encoder.encode(capture.environment.binary_hash),
    encoder.encode(capture.environment.hw_serial),
    encoder.encode(capture.environment.camera_info),
  );

  return keccak256(preimage);
}

async function verifyCaptureOnChain(manifest) {
  if (!appConfig.contractAddress) {
    return {
      available: false,
      verified: false,
      message: "Set VITE_CONTRACT_ADDRESS to enable on-chain verification.",
    };
  }

  const provider = new JsonRpcProvider(appConfig.rpcUrl);
  const contract = new Contract(appConfig.contractAddress, registryAbi, provider);
  const captureHash = computeCapturePrehash(manifest);
  const { v, r, s } = parseCaptureSignature(manifest.signature);
  const scriptHash = sha256StringToBytes32(manifest.environment.script_hash);
  const binaryHash = sha256StringToBytes32(manifest.environment.binary_hash);
  const zeroHash = `0x${"0".repeat(64)}`;

  try {
    const [result, approvedScriptHash, approvedBinaryHash] = await Promise.all([
      contract.verifyCapture(captureHash, v, r, s, scriptHash, binaryHash),
      contract.approvedScriptHash(),
      contract.approvedBinaryHash(),
    ]);

    const scriptConfigured = approvedScriptHash.toLowerCase() !== zeroHash;
    const binaryConfigured = approvedBinaryHash.toLowerCase() !== zeroHash;
    const scriptAccepted = !scriptConfigured || result.scriptMatch;
    const binaryAccepted = !binaryConfigured || result.binaryMatch;

    return {
      available: true,
      verified: result.valid && scriptAccepted && binaryAccepted,
      valid: result.valid,
      signer: result.signer,
      scriptMatch: result.scriptMatch,
      binaryMatch: result.binaryMatch,
      scriptConfigured,
      binaryConfigured,
      captureHash,
      message: result.valid ? "On-chain verification completed." : "Recovered signer is not registered.",
    };
  } catch (error) {
    return {
      available: true,
      verified: false,
      valid: false,
      signer: "",
      scriptMatch: false,
      binaryMatch: false,
      scriptConfigured: false,
      binaryConfigured: false,
      captureHash,
      message: extractError(error),
    };
  }
}

function extractSerialShort(manifest) {
  if (!manifest?.serial) return "";
  const serial = manifest.serial.replace(/\0/g, "").trim();
  return serial.length > 8 ? serial.slice(-8) : serial;
}

function AppContent() {
  const [devices, setDevices] = useState([]);
  const [attesterAddress, setAttesterAddress] = useState("");
  const [networkName, setNetworkName] = useState("");
  const [lastUpdated, setLastUpdated] = useState(null);
  const [loadingDevices, setLoadingDevices] = useState(true);
  const [listError, setListError] = useState("");

  const [walletAddress, setWalletAddress] = useState("");
  const [walletChainId, setWalletChainId] = useState(null);
  const [walletBalance, setWalletBalance] = useState("");
  const [isAuthorizedAttester, setIsAuthorizedAttester] = useState(false);
  const [connectingWallet, setConnectingWallet] = useState(false);

  const [form, setForm] = useState(emptyForm);
  const [submitting, setSubmitting] = useState(false);
  const [txHash, setTxHash] = useState("");
  const [submitError, setSubmitError] = useState("");
  const [showRegister, setShowRegister] = useState(false);
  const [verifyingDemoCapture, setVerifyingDemoCapture] = useState(false);
  const [demoVerification, setDemoVerification] = useState(null);
  const [demoVerificationError, setDemoVerificationError] = useState("");
  const [captureImageFile, setCaptureImageFile] = useState(null);
  const [metadataFile, setMetadataFile] = useState(null);
  const [signFile, setSignFile] = useState(null);
  const [capturePreviewUrl, setCapturePreviewUrl] = useState("");
  const [verifiedManifest, setVerifiedManifest] = useState(null);

  const hasContractAddress = Boolean(appConfig.contractAddress);
  const walletChainMatches =
    walletChainId === null || walletChainId === appConfig.expectedChainId;

  const deviceStats = useMemo(
    () => ({
      total: devices.length,
      active: devices.filter((device) => device.active).length,
    }),
    [devices],
  );

  function clearWalletState() {
    setWalletAddress("");
    setWalletChainId(null);
    setWalletBalance("");
    setIsAuthorizedAttester(false);
  }

  async function loadDevices() {
    if (!hasContractAddress) {
      setListError("Set VITE_CONTRACT_ADDRESS before loading the registry.");
      setDevices([]);
      setLoadingDevices(false);
      return;
    }

    setLoadingDevices(true);
    setListError("");

    try {
      const provider = new JsonRpcProvider(appConfig.rpcUrl);
      const contract = new Contract(appConfig.contractAddress, registryAbi, provider);
      const [network, contractAttester, logs] = await Promise.all([
        provider.getNetwork(),
        contract.ATTESTER(),
        contract.queryFilter(contract.filters.DeviceRegistered(), 0, "latest"),
      ]);

      const uniqueSerialHashes = [...new Set(logs.map((log) => log.args.serialHash))];
      const deviceDetails = await Promise.all(
        uniqueSerialHashes.map(async (serialHash) => {
          const device = await contract.getDevice(serialHash);
          return normalizeRegisteredDevice(serialHash, device);
        }),
      );

      deviceDetails.sort((left, right) => right.attestedAt - left.attestedAt);
      setDevices(deviceDetails.filter((device) => device.active));
      setAttesterAddress(contractAttester);
      setNetworkName(network.name || `Chain ${network.chainId.toString()}`);
      setLastUpdated(new Date());
    } catch (error) {
      setListError(extractError(error));
      setDevices([]);
    } finally {
      setLoadingDevices(false);
    }
  }

  async function updateWalletState(provider, signerAddress) {
    const [network, balance] = await Promise.all([
      provider.getNetwork(),
      provider.getBalance(signerAddress),
    ]);
    const authorized = hasContractAddress
      ? await new Contract(appConfig.contractAddress, registryAbi, provider).isAttester(
          signerAddress,
        )
      : false;

    setWalletAddress(signerAddress);
    setWalletChainId(Number(network.chainId));
    setWalletBalance(Number(formatEther(balance)).toFixed(4));
    setIsAuthorizedAttester(authorized);
  }

  async function connectWallet() {
    if (!window.ethereum) {
      setSubmitError("No wallet found. Install MetaMask or another injected wallet.");
      return;
    }

    setConnectingWallet(true);
    setSubmitError("");

    try {
      const provider = new BrowserProvider(window.ethereum);
      await provider.send("eth_requestAccounts", []);
      const signer = await provider.getSigner();
      const signerAddress = await signer.getAddress();
      await updateWalletState(provider, signerAddress);
    } catch (error) {
      setSubmitError(extractError(error));
    } finally {
      setConnectingWallet(false);
    }
  }

  async function handleSubmit(event) {
    event.preventDefault();
    setSubmitError("");
    setTxHash("");

    if (!hasContractAddress) {
      setSubmitError("Set VITE_CONTRACT_ADDRESS before submitting transactions.");
      return;
    }

    if (!window.ethereum) {
      setSubmitError("No wallet found. Install MetaMask or another injected wallet.");
      return;
    }

    if (!walletAddress) {
      setSubmitError("Connect the authorized attester wallet first.");
      return;
    }

    if (!walletChainMatches) {
      setSubmitError(
        `Switch your wallet to chain ${appConfig.expectedChainId} before submitting.`,
      );
      return;
    }

    if (!isAddress(form.deviceAddress)) {
      setSubmitError("Enter a valid microscope wallet address.");
      return;
    }

    if (!form.serial.trim()) {
      setSubmitError("Serial is required.");
      return;
    }

    setSubmitting(true);

    try {
      const provider = new BrowserProvider(window.ethereum);
      const signer = await provider.getSigner();
      const contract = new Contract(appConfig.contractAddress, registryAbi, signer);
      const serialHash = keccak256(toUtf8Bytes(form.serial.trim()));
      const tx = await contract.registerDevice(serialHash, form.deviceAddress.trim());
      const receipt = await tx.wait();
      setTxHash(receipt.hash);
      setForm(emptyForm);
      await loadDevices();
      await updateWalletState(provider, walletAddress);
    } catch (error) {
      setSubmitError(extractError(error));
    } finally {
      setSubmitting(false);
    }
  }

  async function handleVerifyDemoCapture() {
    setVerifyingDemoCapture(true);
    setDemoVerificationError("");

    try {
      if (!captureImageFile || !metadataFile || !signFile) {
        throw new Error("Choose an image, metadata.json, and sign.json before verifying.");
      }

      const manifest = await parseJsonFile(signFile);
      const expectedFiles = new Map(manifest.files.map((file) => [file.name, file]));

      const actualFiles = await Promise.all([
        hashFile(captureImageFile).then((file) => ({ ...file, name: captureImageFile.name })),
        hashFile(metadataFile).then((file) => ({ ...file, name: metadataFile.name })),
      ]);

      const fileResults = actualFiles.map((file) => {
        const expected = expectedFiles.get(file.name);
        return {
          ...file,
          hashMatches: Boolean(expected) && expected.hash === file.hash,
          sizeMatches: Boolean(expected) && Number(expected.size) === file.size,
        };
      });

      const actualContentHash = await computeContentHash(actualFiles);
      const filesVerified = fileResults.every((file) => file.hashMatches && file.sizeMatches);
      const contentHashMatches = manifest.content_hash === actualContentHash;
      const onChain = await verifyCaptureOnChain(manifest);
      const verified = filesVerified && contentHashMatches && onChain.verified;

      setVerifiedManifest(manifest);
      setDemoVerification({
        verified,
        filesVerified,
        checkedAt: new Date(),
        contentHashMatches,
        actualContentHash,
        expectedContentHash: manifest.content_hash,
        fileResults,
        onChain,
      });
    } catch (error) {
      setDemoVerification(null);
      setVerifiedManifest(null);
      setDemoVerificationError(extractError(error));
    } finally {
      setVerifyingDemoCapture(false);
    }
  }

  function handleCaptureImageChange(event) {
    const nextFile = event.target.files?.[0] || null;
    setDemoVerification(null);
    setDemoVerificationError("");
    setCaptureImageFile(nextFile);
  }

  function handleMetadataChange(event) {
    const nextFile = event.target.files?.[0] || null;
    setDemoVerification(null);
    setDemoVerificationError("");
    setMetadataFile(nextFile);
  }

  function handleSignChange(event) {
    const nextFile = event.target.files?.[0] || null;
    setDemoVerification(null);
    setDemoVerificationError("");
    setSignFile(nextFile);
  }

  useEffect(() => {
    loadDevices();
  }, []);

  useEffect(() => {
    if (!window.ethereum) return;

    async function hydrateWallet() {
      const provider = new BrowserProvider(window.ethereum);
      const accounts = await provider.send("eth_accounts", []);
      if (!accounts.length) return;
      await updateWalletState(provider, accounts[0]);
    }

    hydrateWallet().catch(() => {
      clearWalletState();
    });
  }, []);

  useEffect(() => {
    const injected = window.ethereum;
    if (!injected || typeof injected.on !== "function") return undefined;

    async function handleAccountsChanged(accounts) {
      if (!accounts.length) {
        clearWalletState();
        return;
      }

      const provider = new BrowserProvider(injected);
      await updateWalletState(provider, accounts[0]);
    }

    async function handleChainChanged() {
      await loadDevices();
      if (!walletAddress) return;
      const provider = new BrowserProvider(injected);
      await updateWalletState(provider, walletAddress);
    }

    injected.on("accountsChanged", handleAccountsChanged);
    injected.on("chainChanged", handleChainChanged);

    return () => {
      if (typeof injected.removeListener === "function") {
        injected.removeListener("accountsChanged", handleAccountsChanged);
        injected.removeListener("chainChanged", handleChainChanged);
      }
    };
  }, [walletAddress]);

  useEffect(() => {
    if (!captureImageFile) {
      setCapturePreviewUrl("");
      return undefined;
    }

    const nextUrl = URL.createObjectURL(captureImageFile);
    setCapturePreviewUrl(nextUrl);

    return () => {
      URL.revokeObjectURL(nextUrl);
    };
  }, [captureImageFile]);

  return (
    <div className="shell">
      <header className="topbar">
        <div className="topbar-brand">
          <img className="topbar-logo" src="/assets/biotexturas-logo.png" alt="biotexturas logo" />
          <div className="topbar-text">
            <h1>TerraGenesis</h1>
            <p className="tagline">Provenance for every observation</p>
          </div>
        </div>
        <button className="ghost-button" onClick={connectWallet} disabled={connectingWallet}>
          {connectingWallet ? "Connecting..." : walletAddress ? shortAddress(walletAddress) : "Connect wallet"}
        </button>
      </header>

      <main className="layout">
        {/* --- Hero --- */}
        <section className="hero panel">
          <div className="hero-copy">
            <p className="kicker">Open microscopy meets on-chain provenance</p>
            <h2>Prove your microscopy data is real.</h2>
            <p className="lead">
              Each TerraScope microscope has its own cryptographic identity. Captures are hashed,
              signed, and verifiable on-chain — no intermediaries, no trust required.
            </p>

            <div className="hero-actions">
              <a className="primary-button" href="#verify-capture">
                Verify a capture
              </a>
              <button className="secondary-button" onClick={loadDevices} disabled={loadingDevices}>
                {loadingDevices ? "Refreshing..." : "Refresh registry"}
              </button>
            </div>
          </div>

          <div className="hero-visual">
            <img className="hero-image" src="/assets/DevicePhoto.jpeg" alt="TerraScope microscope" />
            <article className="stat-card">
              <span>Registered TerraScopes</span>
              <strong>{deviceStats.total}</strong>
            </article>
          </div>
        </section>

        {/* --- How it works: 3 inline steps --- */}
        <section className="panel panel-hover">
          <div className="steps-row">
            <div className="step-item">
              <span className="step-icon">
                <img src="/assets/demo-capture/capture.jpg" alt="Microscopy capture" />
              </span>
              <h3>Capture</h3>
              <p>TerraScope takes a biological image</p>
            </div>
            <span className="step-arrow">&rarr;</span>
            <div className="step-item">
              <span className="step-icon">&#x1F512;</span>
              <h3>Sign</h3>
              <p>Device hashes and signs the capture</p>
            </div>
            <span className="step-arrow">&rarr;</span>
            <div className="step-item">
              <span className="step-icon">&#x2713;</span>
              <h3>Verify</h3>
              <p>Anyone checks provenance on-chain</p>
            </div>
          </div>
        </section>

        {/* --- Verify a capture (PRIMARY) --- */}
        <section className="panel demo-verify-panel" id="verify-capture">
          <div className="section-head">
            <div>
              <p className="eyebrow">Verify</p>
              <h2>Upload a capture and verify its provenance.</h2>
            </div>
          </div>

          <div className="demo-verify-grid">
            <div className="demo-preview-card">
              {capturePreviewUrl ? (
                <img
                  className="demo-preview-image"
                  src={capturePreviewUrl}
                  alt="Selected capture preview"
                />
              ) : (
                <div className="demo-preview-empty">
                  <img
                    className="demo-preview-placeholder"
                    src="/assets/demo-capture/capture.jpg"
                    alt="Example TerraScope microscopy capture"
                  />
                </div>
              )}
            </div>

            <div className="demo-verify-copy">
              <p>
                Select your capture image, <code>metadata.json</code>, and <code>sign.json</code>.
              </p>

              <div className="register-form">
                <label>
                  <span>Image file</span>
                  <input type="file" accept="image/*" onChange={handleCaptureImageChange} />
                </label>

                <label>
                  <span>metadata.json</span>
                  <input type="file" accept=".json,application/json" onChange={handleMetadataChange} />
                </label>

                <label>
                  <span>sign.json</span>
                  <input type="file" accept=".json,application/json" onChange={handleSignChange} />
                </label>
              </div>

              <div className="demo-selected-files">
                <span>{captureImageFile ? captureImageFile.name : "No image selected"}</span>
                <span>{metadataFile ? metadataFile.name : "No metadata selected"}</span>
                <span>{signFile ? signFile.name : "No sign file selected"}</span>
              </div>

              <div className="form-actions">
                <button
                  className="primary-button"
                  type="button"
                  onClick={handleVerifyDemoCapture}
                  disabled={
                    verifyingDemoCapture || !captureImageFile || !metadataFile || !signFile
                  }
                >
                  {verifyingDemoCapture ? "Verifying..." : "Verify"}
                </button>
              </div>
            </div>
          </div>

          {demoVerificationError ? <p className="message error">{demoVerificationError}</p> : null}

          {/* --- Big verdict card --- */}
          {demoVerification ? (
            <>
              <div className={`verdict-card ${demoVerification.verified ? "verified" : "not-verified"}`}>
                {demoVerification.verified
                  ? `VERIFIED — from TerraScope #${extractSerialShort(verifiedManifest)}`
                  : "NOT VERIFIED"}
                <div className="verdict-sub">
                  {demoVerification.checkedAt.toLocaleTimeString()}
                </div>
              </div>

              {/* --- Summary row --- */}
              <div className="summary-row">
                <span className={`summary-item ${demoVerification.filesVerified ? "pass" : "fail"}`}>
                  Files {demoVerification.filesVerified ? "\u2713" : "\u2717"}
                </span>
                <span className={`summary-item ${demoVerification.contentHashMatches ? "pass" : "fail"}`}>
                  Content hash {demoVerification.contentHashMatches ? "\u2713" : "\u2717"}
                </span>
                <span className={`summary-item ${demoVerification.onChain.verified ? "pass" : "fail"}`}>
                  On-chain {demoVerification.onChain.verified ? "\u2713" : "\u2717"}
                </span>
              </div>

              {/* --- Expandable technical details --- */}
              <details className="verify-details">
                <summary>Technical details</summary>
                <div className="detail-grid">
                  <div className="detail-row">
                    <dt>Signer</dt>
                    <dd>{demoVerification.onChain.signer || "-"}</dd>
                  </div>
                  <div className="detail-row">
                    <dt>Capture hash</dt>
                    <dd>{demoVerification.onChain.captureHash || "-"}</dd>
                  </div>
                  <div className="detail-row">
                    <dt>Content hash</dt>
                    <dd>{demoVerification.expectedContentHash}</dd>
                  </div>
                  <div className="detail-row">
                    <dt>Registered</dt>
                    <dd>{demoVerification.onChain.valid ? "Yes" : "No"}</dd>
                  </div>
                  <div className="detail-row">
                    <dt>Script hash</dt>
                    <dd>
                      {demoVerification.onChain.scriptConfigured
                        ? demoVerification.onChain.scriptMatch
                          ? "MATCH"
                          : "MISMATCH"
                        : "not configured"}
                    </dd>
                  </div>
                  <div className="detail-row">
                    <dt>Binary hash</dt>
                    <dd>
                      {demoVerification.onChain.binaryConfigured
                        ? demoVerification.onChain.binaryMatch
                          ? "MATCH"
                          : "MISMATCH"
                        : "not configured"}
                    </dd>
                  </div>

                  {demoVerification.fileResults.map((file) => (
                    <div className="detail-row" key={file.name}>
                      <dt>{file.name}</dt>
                      <dd>
                        {file.hashMatches && file.sizeMatches ? "MATCH" : "MISMATCH"} — {file.hash} ({file.size} bytes)
                      </dd>
                    </div>
                  ))}
                </div>
              </details>

              {demoVerification.onChain.message ? (
                <p
                  className={`message ${
                    demoVerification.onChain.verified ? "success" : "error"
                  }`}
                >
                  {demoVerification.onChain.message}
                </p>
              ) : null}
            </>
          ) : null}
        </section>

        {/* --- Registered TerraScopes --- */}
        <section className="panel">
          <div className="section-head">
            <div>
              <p className="eyebrow">Registry</p>
              <h2>TerraScope microscopes registered on-chain.</h2>
            </div>
            <p className="timestamp">
              {lastUpdated ? `Last sync: ${lastUpdated.toLocaleTimeString()}` : "Waiting for first sync"}
            </p>
          </div>

          {listError ? <p className="message error">{listError}</p> : null}

          {loadingDevices ? (
            <div className="empty-state">Loading registry data from the chain...</div>
          ) : devices.length ? (
            <div className="device-grid">
              {devices.map((device) => (
                <article className="device-card" key={device.serialHash}>
                  <div className="device-card-head">
                    <span className="badge success">Registered</span>
                    <span>{formatTimestamp(device.attestedAt)}</span>
                  </div>
                  <h3>{shortAddress(device.deviceAddr)}</h3>
                  <dl>
                    <div>
                      <dt>Serial hash</dt>
                      <dd>{device.serialHash}</dd>
                    </div>
                    <div>
                      <dt>Microscope address</dt>
                      <dd>{device.deviceAddr}</dd>
                    </div>
                    <div>
                      <dt>Attester</dt>
                      <dd>{device.attester}</dd>
                    </div>
                  </dl>
                </article>
              ))}
            </div>
          ) : (
            <div className="empty-state">
              No microscopes are registered yet. Deploy the contract, connect the attester wallet, and
              register the first TerraScope below.
            </div>
          )}
        </section>

        {/* --- Register (attester-only, collapsed) --- */}
        {isAuthorizedAttester ? (
          <section className="panel register-panel">
            {!showRegister ? (
              <div className="register-toggle">
                <button className="secondary-button" onClick={() => setShowRegister(true)}>
                  Register a TerraScope
                </button>
              </div>
            ) : (
              <>
                <div className="section-head">
                  <div>
                    <p className="eyebrow">Register</p>
                    <h2>Register a new TerraScope on-chain.</h2>
                  </div>
                  <span className="badge success">Attester wallet confirmed</span>
                </div>

                <div className="wallet-box">
                  <div>
                    <span>Wallet</span>
                    <strong>{walletAddress ? walletAddress : "No wallet connected"}</strong>
                  </div>
                  <div>
                    <span>Balance</span>
                    <strong>{walletBalance ? `${walletBalance} ETH` : "-"}</strong>
                  </div>
                  <div>
                    <span>Chain</span>
                    <strong>
                      {walletChainId === null ? "-" : walletChainId}
                      {walletChainMatches ? "" : " (wrong chain)"}
                    </strong>
                  </div>
                </div>

                <form className="register-form" onSubmit={handleSubmit}>
                  <label>
                    <span>Microscope serial</span>
                    <input
                      type="text"
                      placeholder="00000000ba807092"
                      value={form.serial}
                      onChange={(event) =>
                        setForm((current) => ({ ...current, serial: event.target.value }))
                      }
                    />
                  </label>

                  <label>
                    <span>Microscope address</span>
                    <input
                      type="text"
                      placeholder="0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"
                      value={form.deviceAddress}
                      onChange={(event) =>
                        setForm((current) => ({ ...current, deviceAddress: event.target.value }))
                      }
                    />
                  </label>

                  <div className="form-actions">
                    <button
                      className="primary-button"
                      type="submit"
                      disabled={submitting || !isAuthorizedAttester || !walletChainMatches}
                    >
                      {submitting ? "Submitting..." : "Register TerraScope"}
                    </button>
                    <button
                      className="secondary-button"
                      type="button"
                      onClick={() => setForm(emptyForm)}
                      disabled={submitting}
                    >
                      Clear form
                    </button>
                    <button
                      className="ghost-button"
                      type="button"
                      onClick={() => setShowRegister(false)}
                    >
                      Close
                    </button>
                  </div>
                </form>

                {submitError ? <p className="message error">{submitError}</p> : null}
                {txHash ? (
                  <p className="message success">
                    Transaction confirmed: <code>{txHash}</code>
                  </p>
                ) : null}
              </>
            )}
          </section>
        ) : null}
      </main>

      {/* --- Footer --- */}
      <footer className="site-footer">
        <img className="footer-logo" src="/assets/biotexturas-logo.png" alt="biotexturas" />
        <span><strong>TerraGenesis</strong> &middot; A biotexturas project &middot; Built on HardTrust</span>
        <span>&middot; Contract: {appConfig.contractAddress || "not configured"} &middot; Chain: {networkName || `${appConfig.expectedChainId}`}</span>
      </footer>
    </div>
  );
}

export default function App() {
  return (
    <AppErrorBoundary>
      <AppContent />
    </AppErrorBoundary>
  );
}
