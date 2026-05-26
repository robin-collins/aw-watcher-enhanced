/**
 * Options page script for ActivityWatch Enhanced browser extension
 */

document.addEventListener('DOMContentLoaded', () => {
  // DOM elements
  const statusDot = document.getElementById('statusDot');
  const statusText = document.getElementById('statusText');
  const serverUrl = document.getElementById('serverUrl');
  const trackUrls = document.getElementById('trackUrls');
  const trackTitles = document.getElementById('trackTitles');
  const incognitoTracking = document.getElementById('incognitoTracking');
  const excludeDomains = document.getElementById('excludeDomains');
  const excludePatterns = document.getElementById('excludePatterns');
  const testConnectionBtn = document.getElementById('testConnection');
  const saveSettingsBtn = document.getElementById('saveSettings');
  const resetSettingsBtn = document.getElementById('resetSettings');
  const statusMessage = document.getElementById('statusMessage');

  // Default settings
  const DEFAULTS = {
    serverUrl: 'http://localhost:5600',
    trackUrls: true,
    trackTitles: true,
    incognitoTracking: false,
    excludeDomains: ['localhost', '127.0.0.1'],
    excludePatterns: [],
  };

  /**
   * Load settings from storage
   */
  function loadSettings() {
    chrome.storage.sync.get(DEFAULTS, (items) => {
      serverUrl.value = items.serverUrl;
      trackUrls.checked = items.trackUrls;
      trackTitles.checked = items.trackTitles;
      incognitoTracking.checked = items.incognitoTracking;
      excludeDomains.value = items.excludeDomains.join('\n');
      excludePatterns.value = items.excludePatterns.join('\n');

      // Test connection after loading
      testConnection();
    });
  }

  /**
   * Save settings to storage
   */
  function saveSettings() {
    const settings = {
      serverUrl: serverUrl.value.trim(),
      trackUrls: trackUrls.checked,
      trackTitles: trackTitles.checked,
      incognitoTracking: incognitoTracking.checked,
      excludeDomains: excludeDomains.value.split('\n').map(s => s.trim()).filter(s => s),
      excludePatterns: excludePatterns.value.split('\n').map(s => s.trim()).filter(s => s),
    };

    chrome.storage.sync.set(settings, () => {
      // Notify background script
      chrome.runtime.sendMessage({ type: 'saveSettings', settings }, () => {
        showStatus('Settings saved successfully!', 'success');
      });
    });
  }

  /**
   * Reset settings to defaults
   */
  function resetSettings() {
    if (confirm('Are you sure you want to reset all settings to defaults?')) {
      chrome.storage.sync.set(DEFAULTS, () => {
        loadSettings();
        showStatus('Settings reset to defaults', 'success');
      });
    }
  }

  /**
   * Test connection to ActivityWatch server
   */
  async function testConnection() {
    statusDot.className = 'status-dot checking';
    statusText.textContent = 'Checking connection...';

    try {
      const response = await fetch(`${serverUrl.value}/api/0/info`, {
        method: 'GET',
        headers: { 'Accept': 'application/json' },
      });

      if (response.ok) {
        const data = await response.json();
        statusDot.className = 'status-dot connected';
        statusText.textContent = `Connected to ${data.hostname || 'ActivityWatch'}`;
      } else {
        throw new Error(`HTTP ${response.status}`);
      }
    } catch (error) {
      statusDot.className = 'status-dot disconnected';
      statusText.textContent = `Not connected: ${error.message}`;
    }
  }

  /**
   * Show status message
   */
  function showStatus(message, type) {
    statusMessage.textContent = message;
    statusMessage.className = `status-message ${type}`;

    setTimeout(() => {
      statusMessage.className = 'status-message';
    }, 3000);
  }

  /**
   * Validate regex patterns
   */
  function validatePatterns() {
    const patterns = excludePatterns.value.split('\n').filter(s => s.trim());
    const invalid = [];

    patterns.forEach((pattern, index) => {
      try {
        new RegExp(pattern);
      } catch (e) {
        invalid.push(`Line ${index + 1}: ${pattern}`);
      }
    });

    if (invalid.length > 0) {
      showStatus(`Invalid regex patterns:\n${invalid.join('\n')}`, 'error');
      return false;
    }

    return true;
  }

  // Event listeners
  testConnectionBtn.addEventListener('click', testConnection);

  saveSettingsBtn.addEventListener('click', () => {
    if (validatePatterns()) {
      saveSettings();
    }
  });

  resetSettingsBtn.addEventListener('click', resetSettings);

  serverUrl.addEventListener('change', testConnection);

  // Load settings on page load
  loadSettings();
});
