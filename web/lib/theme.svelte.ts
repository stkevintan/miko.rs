type Theme = 'light' | 'dark' | 'system';

class ThemeManager {
    #theme = $state<Theme>(
        (localStorage.getItem('theme') as Theme) || 'system',
    );
    #mediaQuery: MediaQueryList | null = null;

    constructor() {
        if (typeof window !== 'undefined') {
            this.#mediaQuery = window.matchMedia(
                '(prefers-color-scheme: dark)',
            );
            this.#mediaQuery.addEventListener('change', () => {
                if (this.#theme === 'system') {
                    this.applyTheme();
                }
            });
        }
        this.applyTheme();
    }

    get theme() {
        return this.#theme;
    }

    setTheme(newTheme: Theme) {
        this.#theme = newTheme;
        localStorage.setItem('theme', newTheme);
        this.applyTheme();
    }

    toggle() {
        // Cycling toggle: light -> dark -> system -> light
        if (this.#theme === 'light') this.setTheme('dark');
        else if (this.#theme === 'dark') this.setTheme('system');
        else this.setTheme('light');
    }

    private applyTheme() {
        if (typeof document !== 'undefined') {
            const isDark =
                this.#theme === 'dark' ||
                (this.#theme === 'system' &&
                    window.matchMedia('(prefers-color-scheme: dark)').matches);

            if (isDark) {
                document.documentElement.classList.add('dark');
            } else {
                document.documentElement.classList.remove('dark');
            }
        }
    }
}

export const themeManager = new ThemeManager();
