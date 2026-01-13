type ToastType = 'success' | 'error' | 'info';

interface Toast {
    id: number;
    message: string;
    type: ToastType;
}

class ToastManager {
    #toasts = $state<Toast[]>([]);
    #counter = 0;

    get toasts() {
        return this.#toasts;
    }

    add(message: string, type: ToastType = 'info', duration = 3000) {
        const id = this.#counter++;
        this.#toasts = [...this.#toasts, { id, message, type }];

        if (duration > 0) {
            setTimeout(() => {
                this.remove(id);
            }, duration);
        }
    }

    success(message: string, duration = 3000) {
        this.add(message, 'success', duration);
    }

    error(message: string, duration = 5000) {
        this.add(message, 'error', duration);
    }

    remove(id: number) {
        this.#toasts = this.#toasts.filter(t => t.id !== id);
    }
}

export const toast = new ToastManager();
