// Language preference persistence
document.addEventListener('DOMContentLoaded', function() {
  // Load saved language preference
  const savedLang = localStorage.getItem('preferredLanguage');
  if (savedLang) {
    activateLanguageTab(savedLang);
  }
  
  // Listen for tab changes and save preference
  document.querySelectorAll('.tabbed-set input[type="radio"]').forEach(function(input) {
    input.addEventListener('change', function() {
      if (this.checked) {
        const label = this.nextElementSibling;
        const langName = label.textContent.trim();
        localStorage.setItem('preferredLanguage', langName);
        
        // Sync all tabs with the same language across the page
        syncLanguageTabs(langName);
      }
    });
  });
  
  // Add keyboard shortcuts for language switching
  document.addEventListener('keydown', function(e) {
    // Alt+1: Pseudocode, Alt+2: Rust, Alt+3: C++, Alt+4: C#
    if (e.altKey && e.key >= '1' && e.key <= '4') {
      e.preventDefault();
      const languages = ['Pseudocode', 'Rust', 'C++', 'C#'];
      const langIndex = parseInt(e.key) - 1;
      if (langIndex < languages.length) {
        activateLanguageTab(languages[langIndex]);
        syncLanguageTabs(languages[langIndex]);
      }
    }
  });
});

function activateLanguageTab(langName) {
  document.querySelectorAll('.tabbed-set input[type="radio"]').forEach(function(input) {
    const label = input.nextElementSibling;
    if (label && label.textContent.includes(langName)) {
      input.checked = true;
    }
  });
}

function syncLanguageTabs(langName) {
  document.querySelectorAll('.tabbed-set').forEach(function(tabSet) {
    const inputs = tabSet.querySelectorAll('input[type="radio"]');
    inputs.forEach(function(input) {
      const label = input.nextElementSibling;
      if (label && label.textContent.includes(langName)) {
        input.checked = true;
      }
    });
  });
}

// Add copy button feedback
document.addEventListener('click', function(e) {
  if (e.target.closest('.md-clipboard')) {
    const button = e.target.closest('.md-clipboard');
    const originalTitle = button.getAttribute('title');
    
    button.setAttribute('title', 'Copied!');
    button.classList.add('md-clipboard--copied');
    
    setTimeout(function() {
      button.setAttribute('title', originalTitle);
      button.classList.remove('md-clipboard--copied');
    }, 2000);
  }
});

// Add smooth scrolling for anchor links
document.addEventListener('click', function(e) {
  const anchor = e.target.closest('a[href^="#"]');
  if (anchor && anchor.getAttribute('href').length > 1) {
    e.preventDefault();
    const targetId = anchor.getAttribute('href').substring(1);
    const targetElement = document.getElementById(targetId);
    
    if (targetElement) {
      targetElement.scrollIntoView({
        behavior: 'smooth',
        block: 'start'
      });
      
      // Update URL without jumping
      if (history.pushState) {
        history.pushState(null, null, '#' + targetId);
      }
    }
  }
});

// Language selector component (if using custom selector)
class LanguageSelector {
  constructor(container) {
    this.container = container;
    this.languages = [
      { name: 'Pseudocode', code: 'pseudo', icon: '📝' },
      { name: 'Rust', code: 'rust', icon: '🦀' },
      { name: 'C++', code: 'cpp', icon: '⚙️' },
      { name: 'C#', code: 'csharp', icon: '🎮' }
    ];
    this.currentLang = localStorage.getItem('preferredLanguage') || 'Rust';
    this.render();
  }
  
  render() {
    const buttons = this.languages.map(lang => {
      const isActive = lang.name === this.currentLang;
      return `
        <button 
          class="language-btn ${isActive ? 'active' : ''}"
          data-lang="${lang.code}"
          aria-label="Switch to ${lang.name}">
          <span class="icon">${lang.icon}</span>
          ${lang.name}
        </button>
      `;
    }).join('');
    
    this.container.innerHTML = `
      <div class="language-selector">
        ${buttons}
      </div>
    `;
    
    this.attachEventListeners();
  }
  
  attachEventListeners() {
    this.container.querySelectorAll('.language-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const langCode = e.currentTarget.getAttribute('data-lang');
        const langName = e.currentTarget.textContent.trim();
        this.setLanguage(langName);
      });
    });
  }
  
  setLanguage(langName) {
    this.currentLang = langName;
    localStorage.setItem('preferredLanguage', langName);
    syncLanguageTabs(langName);
    this.render();
  }
}

// Initialize language selectors
document.addEventListener('DOMContentLoaded', function() {
  document.querySelectorAll('[data-language-selector]').forEach(container => {
    new LanguageSelector(container);
  });
});

// Add progress tracking for learning paths
class ProgressTracker {
  constructor() {
    this.storageKey = 'learningProgress';
    this.progress = this.load();
  }
  
  load() {
    const saved = localStorage.getItem(this.storageKey);
    return saved ? JSON.parse(saved) : {};
  }
  
  save() {
    localStorage.setItem(this.storageKey, JSON.stringify(this.progress));
  }
  
  markComplete(pageId) {
    this.progress[pageId] = {
      completed: true,
      timestamp: new Date().toISOString()
    };
    this.save();
    this.updateUI();
  }
  
  isComplete(pageId) {
    return this.progress[pageId]?.completed || false;
  }
  
  updateUI() {
    document.querySelectorAll('[data-progress-item]').forEach(item => {
      const pageId = item.getAttribute('data-progress-item');
      if (this.isComplete(pageId)) {
        item.classList.add('completed');
      }
    });
  }
}

// Initialize progress tracking
const progressTracker = new ProgressTracker();
document.addEventListener('DOMContentLoaded', function() {
  progressTracker.updateUI();
  
  // Add completion buttons
  document.querySelectorAll('[data-mark-complete]').forEach(btn => {
    btn.addEventListener('click', function() {
      const pageId = this.getAttribute('data-mark-complete');
      progressTracker.markComplete(pageId);
    });
  });
});

// Search enhancements
document.addEventListener('DOMContentLoaded', function() {
  const searchInput = document.querySelector('.md-search__input');
  if (searchInput) {
    // Add search suggestions
    searchInput.addEventListener('input', function() {
      const query = this.value.toLowerCase();
      if (query.length < 2) return;
      
      // You can add custom search logic here
    });
  }
});

// Code block enhancements
document.addEventListener('DOMContentLoaded', function() {
  // Add line highlighting on hover
  document.querySelectorAll('.highlight').forEach(block => {
    const lines = block.querySelectorAll('.linenos, code > span');
    lines.forEach(line => {
      line.addEventListener('mouseenter', function() {
        this.classList.add('highlight-line');
      });
      line.addEventListener('mouseleave', function() {
        this.classList.remove('highlight-line');
      });
    });
  });
});

// Theme toggle enhancement
document.addEventListener('DOMContentLoaded', function() {
  const themeToggle = document.querySelector('[data-md-component="palette"]');
  if (themeToggle) {
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme) {
      document.body.setAttribute('data-md-color-scheme', savedTheme);
    }
    
    // Listen for theme changes
    const observer = new MutationObserver(function(mutations) {
      mutations.forEach(function(mutation) {
        if (mutation.attributeName === 'data-md-color-scheme') {
          const theme = document.body.getAttribute('data-md-color-scheme');
          localStorage.setItem('theme', theme);
        }
      });
    });
    
    observer.observe(document.body, { attributes: true });
  }
});

// Add table of contents scroll spy
document.addEventListener('DOMContentLoaded', function() {
  const tocLinks = document.querySelectorAll('.md-nav--secondary a');
  const headings = document.querySelectorAll('h2, h3');
  
  if (tocLinks.length && headings.length) {
    const observer = new IntersectionObserver(entries => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          const id = entry.target.id;
          tocLinks.forEach(link => {
            if (link.getAttribute('href') === '#' + id) {
              link.classList.add('active');
            } else {
              link.classList.remove('active');
            }
          });
        }
      });
    }, {
      rootMargin: '-100px 0px -66%',
      threshold: 0
    });
    
    headings.forEach(heading => {
      if (heading.id) observer.observe(heading);
    });
  }
});

// Print friendly
window.addEventListener('beforeprint', function() {
  // Expand all tabs for printing
  document.querySelectorAll('.tabbed-set input[type="radio"]').forEach(input => {
    input.checked = true;
  });
});
