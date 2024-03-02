<script>
	let q = 'fuck'
	let hits = []
	async function search() {
		let response = await fetch ("https://webapp.nixrs.ru/indexes/writeups/search", {
			method: 'POST',
			headers: {
				'Content-type': 'application/json',
			},
			body: JSON.stringify({
				q: q,
				limit: 10,
			})
		});
		let json = await response.json();
		hits = json.hits;
	}
</script>

<main>
	<input bind:value={q} />
	<button type="button" on:click={search}>
		Search
	</button>
	<p>
		Result:
	</p>
	<table>
		<tr>
			<th>Team</th>
			<th>Author</th>
			<th>Event</th>
		</tr>
		{#each hits as hit}
		<tr>
			<th>{hit.team}</th>
			<th>{hit.author}</th>
			<th>{hit.event}</th>
		</tr>
		{/each}
	</table>
</main>
