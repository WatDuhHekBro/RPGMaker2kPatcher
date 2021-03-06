// Data Types //
// Int: A 32 bit number of up to 5 digits.
// IntStatic: An Int with a pre-defined length of how many digits it has.
// Int8: An 8 bit number of exactly 1 digit.
// Int16: A 16 bit number of exactly 2 digits, little-endian.
// String: An array of Int8s which form text.

// List //
// Header (only at the top): Int length and String.
// Array: Int id, Int length, and an Int8Array of length bytes.
// Ending Int8 0 as the id is read.

// Object //
// Int number of objects.
// Int id of object.
// Array: Int id, Int length, Int8Array/String/IntArray (Int8Array unless otherwise specified).
// If the length is 0, then the value is null.

class ByteReader
{
	constructor(start, end)
	{
		this.pos = start;
		this.end = end;
	}
	
	static setBytes(b)
	{
		this.bytes = b;
	}
	
	check()
	{
		if(this.pos > this.end)
			throw `FATAL ERROR: Position ${this.pos} exceeded the given endpoint of ${this.end}!`;
	}
	
	spliceBytes()
	{
		this.check();
		let length = this.readInt();
		let reader = new ByteReader(this.pos, this.pos + length);
		this.pos += length;
		return reader;
	}
	
	readMap()
	{
		this.check();
		let values = [];
		
		for(let length = this.readInt()/2; length--;)
			values.push(this.readInt16());
		
		return values;
	}
	
	readCommand()
	{
		this.check();
		return [this.readInt(), this.readInt(), this.readString(), this.readIntArray()];
	}
	
	// Defined Length
	readString()
	{
		this.check();
		let s = '';
		
		for(let length = this.readInt(); length--;)
			s += String.fromCharCode(this.readInt8());
		
		return s;
	}
	
	// Defined Length
	readIntArray()
	{
		this.check();
		let array = [];
		
		for(let length = this.readInt(); length--;)
			array.push(this.readInt());
		
		return array;
	}
	
	// Defined Length
	readInt8Array()
	{
		this.check();
		let array = [];
		
		for(let length = this.readInt(); length--;)
			array.push(this.readInt8());
		
		return array;
	}
	
	// Defined Length
	readIntStatic()
	{
		this.check();
		let digits = [];
		
		for(let length = this.readInt(); length--;)
			digits.push(this.readInt8());
		
		return ByteReader.convertNumber(digits);
	}
	
	readInt()
	{
		this.check();
		let digits = [];
		
		for(let length = 5; length--;)
		{
			let d = ByteReader.bytes[this.pos++];
			digits.push(d);
			
			if(d < 0x80)
				break;
		}
		
		return ByteReader.convertNumber(digits);
	}
	
	readInt16()
	{
		this.check();
		let d1 = ByteReader.bytes[this.pos++];
		let d2 = ByteReader.bytes[this.pos++];
		return (d2 << 8) + d1;
	}
	
	readInt8()
	{
		this.check();
		return ByteReader.bytes[this.pos++];
	}
	
	static convertNumber(digits)
	{
		let sum = 0;
		
		for(let d = digits.length-1, power = 0; d >= 0; d--)
		{
			if(power === 5)
				break;
			
			let num = digits[d];
			num = num >= 0x80 ? (num - 0x80) : num;
			
			if(power === 4 && num >= 0x8)
				sum = (sum + num * Math.pow(0x80, power++)) - 0x100000000;
			else
				sum += num * Math.pow(0x80, power++);
		}
		
		return sum;
	}
}

class ByteWriter
{
	constructor()
	{
		this.bytes = [];
	}
	
	appendBytes(b)
	{
		this.bytes = this.bytes.concat(b);
	}
	
	writeMap(tiles)
	{
		this.writeInt(tiles.length * 2)
		
		for(let tile of tiles)
			this.writeInt16(tile);
	}
	
	writeCommand(e)
	{
		this.writeInt(e[0]);
		this.writeInt(e[1]);
		this.writeString(e[2]);
		this.writeIntArray(e[3]);
	}
	
	// Defined Length
	// Even though my system locale is English (US), it seems that RPG Maker 2k was using Windows-1251 to encode text, which would explain why letters with umlauts appeared as Cyrillic characters.
	writeString(str)
	{
		this.writeInt(str.length);
		
		for(let i = 0, len = str.length; i < len; i++)
		{
			let c = str.charCodeAt(i);
			
			if(c >= 0x410 && c <= 0x44F)
				c -= 0x350;
			else if(c === 0x401)
				c = 0xA8;
			else if(c === 0x451)
				c = 0xB8;
			
			if(c > 0xFF)
			{
				console.warn(`ATTENTION! The character ${c.toString(16).toUpperCase()} is out of bounds!`);
				c = 0x2E; // "."
			}
			
			this.writeInt8(c);
		}
	}
	
	// Defined Length
	writeIntArray(array)
	{
		this.writeInt(array.length);
		
		for(let num of array)
			this.writeInt(num);
	}
	
	// Defined Length
	writeInt8Array(array)
	{
		this.writeInt(array.length);
		this.appendBytes(array);
	}
	
	// Defined Length
	writeIntStatic(num)
	{
		let digits = ByteWriter.convertNumber(num);
		this.writeInt(digits.length);
		this.appendBytes(digits);
	}
	
	writeInt(num)
	{
		this.appendBytes(ByteWriter.convertNumber(num));
	}
	
	writeInt16(num)
	{
		num = num.toString(16).padStart(4, '0');
		this.writeInt8(parseInt(num.slice(2,4), 16));
		this.writeInt8(parseInt(num.slice(0,2), 16));
	}
	
	writeInt8(num)
	{
		this.bytes.push(num);
	}
	
	static convertNumber(num = 0)
	{
		let digits = [0x80, 0x80, 0x80, 0x80, 0x0];
		let negative = num < 0;
		
		if(negative)
		{
			num += 0x100000000;
			
			if(Math.floor(num / 0x80000000) > 1)
				throw `Number ${num} is out of the range of a signed 32 bit integer!`;
			
			num %= 0x80000000;
			digits[0] += 0x8;
		}
		
		for(let d = 0, power = 4; d < 5; d++)
		{
			let p = Math.pow(0x80, power--);
			digits[d] += Math.floor(num / p);
			num %= p;
		}
		
		for(let d = 0; d < 5; d++)
		{
			if(digits[d] !== 0x80)
			{
				digits = digits.slice(d);
				break;
			}
		}
		
		return digits;
	}
}