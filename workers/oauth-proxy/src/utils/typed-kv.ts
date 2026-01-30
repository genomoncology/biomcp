import { GenericSchema, InferOutput, parse } from 'valibot';

export class TypedKV<TSchema extends GenericSchema> {
  static create<TSchema extends GenericSchema>(prefix: string, schema: TSchema, defaultOptions?: KVNamespacePutOptions) {
    return (kv: KVNamespace) => new TypedKV(kv, prefix, schema, defaultOptions);
  }

  constructor(
    private kv: KVNamespace,
    private prefix: string,
    private validator: TSchema,
    private defaultOptions?: KVNamespacePutOptions,
  ) {}

  private getKey(key: string): string {
    return `${this.prefix}:${key}`;
  }

  async put(key: string, value: InferOutput<TSchema>, options: KVNamespacePutOptions = this.defaultOptions ?? {}): Promise<void> {
    const data = parse(this.validator, value);
    await this.kv.put(this.getKey(key), JSON.stringify(data), options);
  }

  async get(key: string): Promise<InferOutput<TSchema> | null> {
    const raw = await this.kv.get(this.getKey(key), 'json');
    if (raw === null) return null;
    return parse(this.validator, raw);
  }

  async delete(key: string): Promise<void> {
    await this.kv.delete(this.getKey(key));
  }
}
