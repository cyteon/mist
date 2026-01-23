<script lang="ts">
    import { onMount } from "svelte";
    import { marked } from "marked";
    import "./docs.css";

    export let data;
    $: {
        data.slug;
        fetchMarkdown();
    }

    let markdown: string = "loading...";

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
    <div class="w-full flex overflow-auto min-h-screen flex-col">
        <p class="prose prose-lg px-4 py-4 xl:px-64 md:py-8 min-w-full">
            {@html markdown}
        </p>

        <p class="pb-7 mx-auto mt-auto">End of Page</p>
    </div>
</div>