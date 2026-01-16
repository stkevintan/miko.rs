<script lang="ts" generics="T">
    import { type Snippet } from 'svelte';
    import { tick } from 'svelte';
    import { Loader2 } from 'lucide-svelte';

    let {
        data,
        loading = false,
        header,
        row,
        emptyState,
        minWidth = '800px',
        fixed = false,
        resizable = false,
    }: {
        data: T[];
        loading?: boolean;
        header: Snippet;
        row: Snippet<[T]>;
        emptyState?: Snippet;
        minWidth?: string;
        fixed?: boolean;
        resizable?: boolean;
    } = $props();

    // resizable columns logic from http://dobtco.github.io/jquery-resizable-columns/
    let container = $state<HTMLDivElement>();
    let table = $state<HTMLTableElement>();
    let handleContainer = $state<HTMLDivElement>();

    let teardownResize: (() => void) | undefined;

    const UNRESIZABLE_CLASS = 'rc-unresizable';
    const MIN_WIDTH = 0.01;

    type ResizeOperation = {
        left: HTMLTableCellElement;
        right: HTMLTableCellElement;
        handle: HTMLDivElement;
        startX: number;
        widths: { left: number; right: number };
        newWidths: { left: number; right: number };
    };

    let operation: ResizeOperation | null = null;

    const isUnresizable = (cell: HTMLTableCellElement | undefined | null) =>
        !cell || cell.classList.contains(UNRESIZABLE_CLASS);

    const getHeaders = () => {
        if (!table) return [] as HTMLTableCellElement[];
        if (table.querySelector('thead')) {
            return Array.from(
                table.querySelectorAll('thead th'),
            ) as HTMLTableCellElement[];
        }
        return Array.from(
            table.querySelectorAll('tbody tr:first-child td'),
        ) as HTMLTableCellElement[];
    };

    const parseWidth = (cell: HTMLTableCellElement) => {
        const value = parseFloat(cell.style.width.replace('%', ''));
        if (!Number.isNaN(value)) return value;
        const tableWidth = table?.getBoundingClientRect().width ?? 0;
        if (tableWidth <= 0) return 0;
        return (cell.getBoundingClientRect().width / tableWidth) * 100;
    };

    const setWidth = (cell: HTMLTableCellElement, width: number) => {
        const next = Math.max(0, width).toFixed(2);
        cell.style.width = `${next}%`;
    };

    const constrainWidth = (width: number) => {
        if (Number.isNaN(width)) return MIN_WIDTH;
        return Math.max(MIN_WIDTH, width);
    };

    const assignPercentageWidths = (headers: HTMLTableCellElement[]) => {
        if (!table) return;
        const tableWidth = table.getBoundingClientRect().width;
        if (tableWidth <= 0) return;
        for (const header of headers) {
            const width =
                (header.getBoundingClientRect().width / tableWidth) * 100;
            setWidth(header, width);
        }
    };

    const syncHandlePositions = (headers: HTMLTableCellElement[]) => {
        if (!table || !handleContainer) return;
        const tableRect = table.getBoundingClientRect();
        handleContainer.style.width = `${tableRect.width}px`;
        handleContainer.style.height = `${tableRect.height}px`;
        handleContainer.style.left = `${table.offsetLeft}px`;
        handleContainer.style.top = `${table.offsetTop}px`;

        const containerRect = handleContainer.getBoundingClientRect();
        const theadHeight = table.tHead?.getBoundingClientRect().height ?? 0;
        const handleHeight = tableRect.height || theadHeight;

        const handles = Array.from(
            handleContainer.querySelectorAll<HTMLDivElement>('.resize-handle'),
        );

        for (const handle of handles) {
            const index = Number(handle.dataset.index);
            const header = headers[index];
            if (!header) continue;
            const headerRect = header.getBoundingClientRect();
            const left = headerRect.right - containerRect.left;
            handle.style.left = `${left}px`;
            handle.style.height = `${handleHeight}px`;
        }
    };

    const getPointerX = (event: MouseEvent | TouchEvent) => {
        if ('touches' in event) {
            const touch = event.touches[0] ?? event.changedTouches[0];
            return touch?.pageX ?? 0;
        }
        return event.pageX;
    };

    const onPointerMove = (event: MouseEvent | TouchEvent) => {
        if (!operation || !table) return;
        const tableWidth = table.getBoundingClientRect().width;
        if (tableWidth <= 0) return;
        const difference =
            ((getPointerX(event) - operation.startX) / tableWidth) * 100;
        if (difference === 0) return;

        let widthLeft: number | undefined;
        let widthRight: number | undefined;

        if (difference > 0) {
            widthLeft = constrainWidth(
                operation.widths.left +
                    (operation.widths.right - operation.newWidths.right),
            );
            widthRight = constrainWidth(operation.widths.right - difference);
        } else if (difference < 0) {
            widthLeft = constrainWidth(operation.widths.left + difference);
            widthRight = constrainWidth(
                operation.widths.right +
                    (operation.widths.left - operation.newWidths.left),
            );
        }

        if (widthLeft !== undefined) {
            setWidth(operation.left, widthLeft);
            operation.newWidths.left = widthLeft;
        }

        if (widthRight !== undefined) {
            setWidth(operation.right, widthRight);
            operation.newWidths.right = widthRight;
        }

        if (handleContainer) {
            const containerRect = handleContainer.getBoundingClientRect();
            const left =
                operation.left.getBoundingClientRect().right -
                containerRect.left;
            operation.handle.style.left = `${left}px`;
        }
    };

    const onPointerUp = () => {
        if (!operation) return;
        document.body.classList.remove('is-resizing');
        container?.classList.remove('is-resizing');
        window.removeEventListener('mousemove', onPointerMove as EventListener);
        window.removeEventListener('touchmove', onPointerMove as EventListener);
        window.removeEventListener('mouseup', onPointerUp);
        window.removeEventListener('touchend', onPointerUp);
        operation = null;
        syncHandlePositions(getHeaders());
    };

    const onPointerDown = (event: MouseEvent | TouchEvent) => {
        if ('button' in event && event.button !== 0) return;
        if (!table || !handleContainer) return;
        event.preventDefault();

        if (operation) {
            onPointerUp();
        }

        const handle = event.currentTarget as HTMLDivElement | null;
        if (!handle) return;
        const headers = getHeaders();
        const index = Number(handle.dataset.index);
        const left = headers[index];
        const right = headers[index + 1];
        if (!left || !right) return;
        if (isUnresizable(left) || isUnresizable(right)) return;

        const leftWidth = parseWidth(left);
        const rightWidth = parseWidth(right);

        operation = {
            left,
            right,
            handle,
            startX: getPointerX(event),
            widths: { left: leftWidth, right: rightWidth },
            newWidths: { left: leftWidth, right: rightWidth },
        };

        document.body.classList.add('is-resizing');
        container?.classList.add('is-resizing');

        window.addEventListener('mousemove', onPointerMove as EventListener);
        window.addEventListener('touchmove', onPointerMove as EventListener, {
            passive: false,
        });
        window.addEventListener('mouseup', onPointerUp);
        window.addEventListener('touchend', onPointerUp);
    };

    const clearHandles = () => {
        if (!handleContainer) return;
        handleContainer.innerHTML = '';
    };

    const setupResizableHeaders = async () => {
        if (!resizable || !table || !handleContainer) {
            onPointerUp();
            teardownResize?.();
            teardownResize = undefined;
            clearHandles();
            return;
        }

        await tick();

        const headers = getHeaders();
        if (headers.length === 0) return;

        assignPercentageWidths(headers);
        clearHandles();

        const cleanups: Array<() => void> = [];

        headers.forEach((header, index) => {
            const next = headers[index + 1];
            if (!next || isUnresizable(header) || isUnresizable(next)) return;

            const handle = document.createElement('div');
            handle.className = 'resize-handle';
            handle.dataset.index = String(index);
            handle.setAttribute('aria-hidden', 'true');
            handleContainer?.appendChild(handle);

            const onMouseDown = (event: MouseEvent | TouchEvent) =>
                onPointerDown(event);
            handle.addEventListener('mousedown', onMouseDown);
            handle.addEventListener('touchstart', onMouseDown, {
                passive: false,
            });

            cleanups.push(() =>
                handle.removeEventListener('mousedown', onMouseDown),
            );
            cleanups.push(() =>
                handle.removeEventListener('touchstart', onMouseDown),
            );
        });

        const onScroll = () => syncHandlePositions(headers);
        const onResize = () => syncHandlePositions(headers);

        container?.addEventListener('scroll', onScroll, { passive: true });
        window.addEventListener('resize', onResize);
        cleanups.push(() => container?.removeEventListener('scroll', onScroll));
        cleanups.push(() => window.removeEventListener('resize', onResize));

        syncHandlePositions(headers);

        teardownResize?.();
        teardownResize = () => {
            for (const cleanup of cleanups) cleanup();
        };
    };

    $effect(() => {
        void setupResizableHeaders();
        return () => {
            onPointerUp();
            teardownResize?.();
        };
    });

    $effect(() => {
        if (!resizable) return;
        data.length;
        void (async () => {
            await tick();
            syncHandlePositions(getHeaders());
        })();
    });
</script>

<div
    bind:this={container}
    class="flex-1 overflow-auto data-table-container custom-scrollbar relative"
>
    <div
        bind:this={handleContainer}
        class="resize-handle-container"
        aria-hidden="true"
    ></div>
    <table
        bind:this={table}
        class="text-left w-full border-separate border-spacing-0"
        class:table-fixed={fixed}
        style:min-width={minWidth}
        style:table-layout={fixed || resizable ? 'fixed' : undefined}
    >
        <thead
            class="sticky top-0 z-10 bg-white/80 dark:bg-gray-900/80 backdrop-blur-md"
        >
            <tr>
                {@render header()}
            </tr>
        </thead>
        <tbody class="divide-y divide-gray-50 dark:divide-gray-800">
            {#each data as item}
                <tr
                    class="group hover:bg-orange-50/30 dark:hover:bg-orange-500/5 transition-colors cursor-default"
                >
                    {@render row(item)}
                </tr>
            {/each}
        </tbody>
    </table>

    {#if loading}
        <div
            class="absolute inset-0 flex items-center justify-center bg-white/50 dark:bg-gray-900/50 z-20"
        >
            <Loader2 class="animate-spin text-orange-500" size={32} />
        </div>
    {:else if data.length === 0}
        {#if emptyState}
            <div class="flex flex-col items-center justify-center py-20">
                {@render emptyState()}
            </div>
        {/if}
    {/if}
</div>

<style lang="postcss">
    @reference "../../style.css";

    .custom-scrollbar::-webkit-scrollbar {
        width: 6px;
        height: 6px;
    }

    .custom-scrollbar::-webkit-scrollbar-track {
        @apply bg-transparent;
    }

    .custom-scrollbar::-webkit-scrollbar-thumb {
        @apply bg-gray-200 dark:bg-gray-800 rounded-full;
    }

    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        @apply bg-gray-300 dark:bg-gray-700;
    }

    .data-table-container :global(th) {
        @apply px-6 py-4 text-xs font-semibold uppercase tracking-wider text-gray-500 border-b border-r border-gray-100 dark:border-gray-800 relative select-none truncate;
        min-width: 0;
    }

    .data-table-container :global(th:last-child) {
        @apply border-r-0;
    }

    .data-table-container :global(td) {
        @apply overflow-hidden;
        min-width: 0;
        max-width: 0; /* Critical for fixed layout tables to allow cell to shrink below content size */
    }

    .data-table-container :global(td > *:not(.no-truncate)) {
        @apply truncate;
        min-width: 0;
    }

    .data-table-container :global(.resize-handle-container) {
        @apply absolute pointer-events-none z-20;
    }

    .data-table-container :global(.resize-handle) {
        @apply absolute top-0 w-2 cursor-col-resize pointer-events-auto hover:bg-orange-500/50 transition-colors active:bg-orange-600;
        transform: translateX(-50%);
    }

    :global(body.is-resizing) {
        @apply cursor-col-resize select-none;
    }

    .data-table-container.is-resizing :global(tbody) {
        @apply pointer-events-none;
    }

    .data-table-container :global(th:first-child) {
        @apply pl-6;
    }
</style>
