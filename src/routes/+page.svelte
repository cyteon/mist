<script lang="ts">
    const features = [
        { name: "Authentication", status: "complete" },
        { name: "World saving", status: "complete" },
        { name: "Commands", status: "partial" },
        { name: "Chat", status: "complete" },
        { name: "Worldgen", status: "partial" },
        { name: "Survival mode", status: "not_started" },
        { name: "Inventory managment", status: "not_started" },
        { name: "Player sync", status: "not_started" },
        { name: "Entity system", status: "not_started" },
        { name: "The Nether", status: "not_started" },
        { name: "The End", status: "not_started" },
        { name: "Redstone", status: "not_started" }
    ]
</script>

<div class="flex h-screen w-full">
    <nav class="absolute top-0 left-1/2 -translate-x-1/2 mt-4 space-x-2 flex">
        <a href="https://github.com/cyteon/mist" target="_blank" rel="noopener" class="text-neutral-400 hover:text-green-300">source code</a>
        <p class="text-neutral-500">\</p>
        <a href="docs/intro" target="_blank" rel="noopener" class="text-neutral-400 hover:text-green-300">docs</a>
        <p class="text-neutral-500">\</p>
        <a href="https://github.com/cyteon/mist/releases/latest" target="_blank" rel="noopener" class="text-neutral-400 hover:text-green-300">download</a>
    </nav>

    <div class="m-auto text-center px-2">
        <h1 class="text-7xl font-bold text-green-300">Mist</h1>
        <h2 class="text-lg">A (in-progress) reimplementation of the minecraft server in rust</h2>

        <div class="mt-8">
            <h2 class="text-2xl font-bold">Feature progress</h2>

            <div class="grid grid-cols-2 md:grid-cols-3 gap-4 mt-4">
                {#each features as feature}
                    <div class="p-4 rounded-md" class:bg-green-600={feature.status === "complete"} class:bg-yellow-600={feature.status === "partial"} class:bg-red-500={feature.status === "not_started"}>
                        <h3 class="text-lg font-semibold">{feature.name}</h3>
                    </div>
                {/each}
            </div>
        </div>
    </div>
</div>