<script lang="ts">
    import { onMount } from "svelte";
    import { marked } from "marked";
    import { Menu } from "lucide-svelte";

    import docData from "$lib/docs/data.js";
    import "./docs.css";

    export let data;
    $: {
        data.slug;
        fetchMarkdown();
    }

    let markdown: string = "loading...";
    let sidebarOpen: boolean = true;

    async function fetchMarkdown() {
        try {
            await import(`$lib/docs/pages/${data.slug.replace(".md", "")}.md?raw`)
                .then(async (res) => {
                    markdown = await marked(res.default);
                });
        } catch (e) {
            markdown = "404 - Document not found";
        }
    }

    onMount(fetchMarkdown);
</script>

<div class="flex h-screen w-full">
    <div class={`p-2 flex flex-col border-r border-r-neutral-800 ${sidebarOpen ? "w-52" : "w-16"}`}>
        {#each docData.pages as page}
            <a
                href={`/docs/${page.slug}`}
                class="px-2 py-2 flex items-center space-x-2 group"
                class:selected={data.slug === page.slug}
            >
                <page.icon size={24} class="my-auto group-hover:stroke-green-200! group-[.selected]:stroke-green-200!" />
                {#if sidebarOpen}
                    <span class="my-auto group-hover:text-green-200! group-[.selected]:text-green-200!">{page.title}</span>
                {/if}
            </a>
        {/each}

        <button class="mt-auto p-2 rounded-md" on:click={() => sidebarOpen = !sidebarOpen}>
            <Menu size={32} class="my-auto" />
        </button>
    </div>

    <div class="w-full flex overflow-auto min-h-screen flex-col">
        <p class="prose prose-lg px-4 py-4 xl:px-64 md:py-8 min-w-full">
            {@html markdown}
        </p>

        <p class="pb-7 mx-auto mt-auto">End of Page</p>
    </div>
</div>