[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteTag

# Class: NoteTag

## Methods

### asU32()

> **asU32**(): `number`

#### Returns

`number`

***

### executionMode()

> **executionMode**(): [`NoteExecutionMode`](NoteExecutionMode.md)

#### Returns

[`NoteExecutionMode`](NoteExecutionMode.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### isSingleTarget()

> **isSingleTarget**(): `boolean`

#### Returns

`boolean`

***

### forLocalUseCase()

> `static` **forLocalUseCase**(`use_case_id`, `payload`): `NoteTag`

#### Parameters

##### use\_case\_id

`number`

##### payload

`number`

#### Returns

`NoteTag`

***

### forPublicUseCase()

> `static` **forPublicUseCase**(`use_case_id`, `payload`, `execution`): `NoteTag`

#### Parameters

##### use\_case\_id

`number`

##### payload

`number`

##### execution

[`NoteExecutionMode`](NoteExecutionMode.md)

#### Returns

`NoteTag`

***

### fromAccountId()

> `static` **fromAccountId**(`account_id`): `NoteTag`

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`NoteTag`
