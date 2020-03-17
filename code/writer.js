function createStart(data, definitions)
{
	let writer = new ByteWriter();
	writer.writeString(data.header);
	delete data.header;
	stepWrite(data, writer, definitions);
	//writer.writeInt8(0); // I don't know why the database doesn't have that last byte at the end.
	return writer.bytes;
}

function createObject(data, definitions)
{
	let writer = new ByteWriter();
	writer.writeInt(Object.keys(data).length);
	
	for(let id in data)
	{
		writer.writeInt(parseInt(id));
		stepWrite(data[id], writer, definitions);
		writer.writeInt8(0);
	}
	
	return writer;
}

function createList(data, definitions)
{
	let writer = new ByteWriter();
	stepWrite(data, writer, definitions);
	writer.writeInt8(0);
	return writer;
}

function stepWrite(data, writer, definitions)
{
	for(let id in data)
	{
		id = parseInt(id);
		let val = data[id];
		
		if(id !== definitions.commands)
			writer.writeInt(id);
		
		if(val == null)
			writer.writeInt8(0);
		else if('strings' in definitions && (definitions.strings === '*' || definitions.strings.includes(id)))
			writer.writeString(val);
		else if('objects' in definitions && definitions.objects.includes(id))
		{
			let wr = createObject(data[id], definitions.special[id]);
			writer.writeInt(wr.bytes.length);
			writer.appendBytes(wr.bytes);
		}
		else if('lists' in definitions && definitions.lists.includes(id))
		{
			let wr = createList(data[id], definitions.special[id]);
			writer.writeInt(wr.bytes.length);
			writer.appendBytes(wr.bytes);
		}
		else if(id === definitions.commands)
		{
			let wr = new ByteWriter();
			for(let cmd of val)
				wr.writeCommand(cmd);
			wr.writeCommand([0, 0, '', []]);
			
			writer.writeInt(definitions.bytecount);
			writer.writeIntStatic(wr.bytes.length);
			writer.writeInt(id);
			writer.writeInt(wr.bytes.length);
			writer.appendBytes(wr.bytes);
		}
		else
			writer.writeInt8Array(val);
	}
}