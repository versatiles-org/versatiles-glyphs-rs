/**
 * Minimal decoder for SDF glyph PBFs: `glyphs` (field 1: fontstack) → `fontstack`
 * (field 3: glyph) → `glyph`. Returns all glyphs as
 * `{ id, bitmap, width, height, left, top, advance }`, where `bitmap` is a
 * Uint8Array of `(width + 6) × (height + 6)` SDF values, or `null`.
 */
function decodeGlyphPbf(bytes) {
	const glyphs = []
	readFields(bytes, (tag, stack) => {
		if (tag !== 1) return
		readFields(stack, (tag, message) => {
			if (tag !== 3) return
			const glyph = {
				id: 0,
				bitmap: null,
				width: 0,
				height: 0,
				left: 0,
				top: 0,
				advance: 0,
			}
			readFields(message, (tag, value) => {
				if (tag === 1) glyph.id = value
				else if (tag === 2) glyph.bitmap = value
				else if (tag === 3) glyph.width = value
				else if (tag === 4) glyph.height = value
				else if (tag === 5) glyph.left = zigzag(value)
				else if (tag === 6) glyph.top = zigzag(value)
				else if (tag === 7) glyph.advance = value
			})
			glyphs.push(glyph)
		})
	})
	return glyphs
}

/**
 * Calls `callback(tag, value)` for every field of a protobuf message. Varints are
 * passed as numbers, length-delimited fields as Uint8Array views.
 */
function readFields(bytes, callback) {
	let pos = 0
	const readVarint = () => {
		let result = 0
		let shift = 0
		let byte
		do {
			if (pos >= bytes.length) throw new Error('Truncated varint')
			byte = bytes[pos++]
			result += (byte & 0x7f) * 2 ** shift
			shift += 7
		} while (byte & 0x80)
		return result
	}
	while (pos < bytes.length) {
		const key = readVarint()
		const tag = Math.floor(key / 8)
		const type = key % 8
		if (type === 0) {
			callback(tag, readVarint())
		} else if (type === 2) {
			const length = readVarint()
			if (pos + length > bytes.length) throw new Error('Truncated field')
			callback(tag, bytes.subarray(pos, pos + length))
			pos += length
		} else {
			throw new Error(`Unsupported wire type ${type}`)
		}
	}
}

/** Decodes a zigzag-encoded `sint32`. */
function zigzag(n) {
	return n % 2 ? -(n + 1) / 2 : n / 2
}

if (typeof module !== 'undefined') module.exports = { decodeGlyphPbf }
