/**
 * I want to create a system where patching is more manual and works at the binary level.
 * The reason for this, in stark contrast to maps, is to patch the database incrementally.
 * Instead of expanding the entire database to a massive size just to modify one scene,
 * I want to modify only as I need to.
 * So there won't be a single database JSON, but rather, many separate patches. Likewise,
 * there'll need to be many separate functions per database type.
 */
// path: [global_id, sub_id, ...]
// patch: ...

// Enter the bytes of the database and a JSON patch, it'll download the patched database.
/*function applyPatchDatabase(database, patch)
{
	let reader = new ByteReader(database);
	let timeout = 50;
	let id = reader.readInt();
	let length = reader.readInt();
	reader.readString();
	
	while(id !== patch.path[0] && timeout > 0)
	{
		reader.pos += length;
		id = reader.readInt();
		length = reader.readInt();
		timeout--;
	}
	
	timeout = 50;
	
	return reader;
}*/

// Common Events //
function applyPatchDBCommon(database, patch)
{
	
}