// Custom JavaScript for Vela documentation

(function () {
    'use strict';

    // Add "Copy" button to code blocks
    function addCopyButtons() {
        const codeBlocks = document.querySelectorAll('pre > code');

        codeBlocks.forEach((codeBlock) => {
            const pre = codeBlock.parentElement;
            const button = document.createElement('button');
            button.className = 'copy-button';
            button.textContent = 'Copy';
            button.style.position = 'absolute';
            button.style.top = '8px';
            button.style.right = '8px';
            button.style.padding = '4px 8px';
            button.style.fontSize = '12px';
            button.style.border = '1px solid #ddd';
            button.style.borderRadius = '4px';
            button.style.backgroundColor = '#fff';
            button.style.cursor = 'pointer';
            button.style.opacity = '0';
            button.style.transition = 'opacity 0.2s';

            pre.style.position = 'relative';
            pre.appendChild(button);

            pre.addEventListener('mouseenter', () => {
                button.style.opacity = '1';
            });

            pre.addEventListener('mouseleave', () => {
                button.style.opacity = '0';
            });

            button.addEventListener('click', async () => {
                try {
                    await navigator.clipboard.writeText(codeBlock.textContent);
                    button.textContent = 'Copied!';
                    setTimeout(() => {
                        button.textContent = 'Copy';
                    }, 2000);
                } catch (err) {
                    console.error('Failed to copy:', err);
                    button.textContent = 'Failed';
                    setTimeout(() => {
                        button.textContent = 'Copy';
                    }, 2000);
                }
            });
        });
    }

    // Highlight current section in sidebar
    function highlightCurrentSection() {
        const chapters = document.querySelectorAll('.chapter');
        const currentPath = window.location.pathname;

        chapters.forEach((chapter) => {
            const link = chapter.querySelector('a');
            if (link && link.getAttribute('href') === currentPath) {
                chapter.classList.add('expanded');
            }
        });
    }

    // Add external link icon
    function addExternalLinkIcons() {
        const links = document.querySelectorAll('.content a');

        links.forEach((link) => {
            if (link.hostname && link.hostname !== window.location.hostname) {
                link.setAttribute('target', '_blank');
                link.setAttribute('rel', 'noopener noreferrer');
                link.insertAdjacentHTML('beforeend', ' <span style="font-size: 0.8em;">↗</span>');
            }
        });
    }

    // Add version selector (placeholder)
    function addVersionSelector() {
        const menuBar = document.querySelector('.menu-bar');
        if (!menuBar) return;

        const versionDiv = document.createElement('div');
        versionDiv.className = 'version-selector';
        versionDiv.style.display = 'inline-block';
        versionDiv.style.marginLeft = '1rem';
        versionDiv.innerHTML = '<span style="color: #666;">v0.1.0 (Pre-Alpha)</span>';

        const rightButtons = menuBar.querySelector('.right-buttons');
        if (rightButtons) {
            rightButtons.insertBefore(versionDiv, rightButtons.firstChild);
        }
    }

    // Smooth scroll for anchor links
    function enableSmoothScroll() {
        document.querySelectorAll('a[href^="#"]').forEach((anchor) => {
            anchor.addEventListener('click', function (e) {
                e.preventDefault();
                const target = document.querySelector(this.getAttribute('href'));
                if (target) {
                    target.scrollIntoView({
                        behavior: 'smooth',
                        block: 'start'
                    });
                }
            });
        });
    }

    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    function init() {
        addCopyButtons();
        highlightCurrentSection();
        addExternalLinkIcons();
        addVersionSelector();
        enableSmoothScroll();
    }

    // Re-initialize on page navigation (for single-page navigation)
    window.addEventListener('popstate', init);
})();
