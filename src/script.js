// Financial Review Dashboard - Main JavaScript

// Import Tauri API
const { invoke } = window.__TAURI__.tauri;
const { open, save } = window.__TAURI__.dialog;

// Global Application State
let appData = {
    households: [],
    settings: {
        lastFilePath: null,
        theme: 'light',
        autoBackup: true,
        backupCount: 10
    },
    version: '1.0.0'
};

let currentFilePath = null;
let nextHouseholdId = 1;
let selectedHouseholds = new Set();
let isDirty = false;
let chart = null;

// Constants
const MONTHS = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December'
];

const SEGMENTS = ['Black', 'Green', 'Yellow', 'Red'];
const REVIEW_TYPES = ['Required', 'Periodic'];
const STATUSES = ['Scheduled', 'Completed', 'Overdue'];
const PRIORITIES = ['Standard', 'VIP', 'High'];

// Initialize Application
document.addEventListener('DOMContentLoaded', async () => {
    await initializeApp();
    setupEventListeners();
    await loadDefaultData();
    updateUI();
});

// Initialize Application
async function initializeApp() {
    // Detect system theme preference
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    if (!appData.settings.theme) {
        appData.settings.theme = prefersDark ? 'dark' : 'light';
    }
    applyTheme(appData.settings.theme);

    // Listen for system theme changes
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
        if (!document.documentElement.getAttribute('data-theme')) {
            applyTheme(e.matches ? 'dark' : 'light');
        }
    });

    // Initialize Chart.js with theme support
    Chart.defaults.font.family = getComputedStyle(document.documentElement).getPropertyValue('--font-family');
    Chart.defaults.responsive = true;
    Chart.defaults.maintainAspectRatio = false;
}

// Setup Event Listeners
function setupEventListeners() {
    // File operations
    document.getElementById('newFileBtn').addEventListener('click', createNewFile);
    document.getElementById('openFileBtn').addEventListener('click', openFile);
    document.getElementById('saveAsBtn').addEventListener('click', saveAsFile);
    document.getElementById('backupBtn').addEventListener('click', createBackup);
    document.getElementById('exportBtn').addEventListener('click', exportToCSV);

    // Theme toggle
    document.getElementById('themeToggle').addEventListener('click', toggleTheme);

    // View toggle
    document.getElementById('gridViewBtn').addEventListener('click', () => switchView('grid'));
    document.getElementById('tableViewBtn').addEventListener('click', () => switchView('table'));

    // Household management
    document.getElementById('addHouseholdBtn').addEventListener('click', () => openHouseholdModal());
    document.getElementById('householdForm').addEventListener('submit', saveHousehold);
    document.getElementById('cancelBtn').addEventListener('click', closeHouseholdModal);
    document.getElementById('addPersonBtn').addEventListener('click', addPersonEntry);

    // Search and filter
    document.getElementById('searchInput').addEventListener('input', debounce(filterTable, 300));
    document.getElementById('filterSelect').addEventListener('change', filterTable);

    // Table selection
    document.getElementById('selectAll').addEventListener('change', toggleSelectAll);

    // Bulk actions
    document.getElementById('bulkCompleteBtn').addEventListener('click', bulkMarkCompleted);
    document.getElementById('bulkAssignBtn').addEventListener('click', bulkAssignMonth);
    document.getElementById('bulkDeleteBtn').addEventListener('click', bulkDeleteHouseholds);

    // Review completion modal
    document.getElementById('confirmCompleteBtn').addEventListener('click', confirmReviewCompletion);
    document.getElementById('cancelCompleteBtn').addEventListener('click', closeReviewCompleteModal);
    
    // Review advance radio buttons
    document.querySelectorAll('input[name="reviewAdvance"]').forEach(radio => {
        radio.addEventListener('change', toggleCustomDateField);
    });

    // Modal close buttons
    document.querySelectorAll('.modal-close').forEach(btn => {
        btn.addEventListener('click', (e) => {
            const modal = e.target.closest('.modal');
            modal.classList.remove('active');
        });
    });

    // Stats card filters
    document.querySelectorAll('.stat-card.clickable').forEach(card => {
        card.addEventListener('click', (e) => {
            const filter = e.currentTarget.dataset.filter;
            applyStatsFilter(filter);
        });
    });

    // Keyboard shortcuts
    document.addEventListener('keydown', handleKeyboardShortcuts);

    // Auto-save on data changes
    document.addEventListener('dataChanged', debounce(autoSave, 1000));

    // Prevent data loss on page unload
    window.addEventListener('beforeunload', (e) => {
        if (isDirty) {
            e.preventDefault();
            e.returnValue = '';
        }
    });
}

// Theme Management
function toggleTheme() {
    const currentTheme = document.documentElement.getAttribute('data-theme') || 'light';
    const newTheme = currentTheme === 'light' ? 'dark' : 'light';
    applyTheme(newTheme);
    appData.settings.theme = newTheme;
    markDirty();
    
    // Update theme icon
    const themeIcon = document.querySelector('.theme-icon');
    th
