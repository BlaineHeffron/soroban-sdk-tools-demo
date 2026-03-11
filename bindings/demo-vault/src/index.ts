import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}




export const VaultError = {
  /**
   * vault already initialized
   */
  1: {message:"AlreadyInitialized"},
  /**
   * vault not initialized
   */
  2: {message:"NotInitialized"},
  /**
   * deposit amount must be positive
   */
  3: {message:"InvalidAmount"},
  /**
   * withdrawal exceeds deposit
   */
  4: {message:"InsufficientDeposit"},
  /**
   * already initialized
   */
  5: {message:"Token_AlreadyInitialized"},
  /**
   * not initialized
   */
  6: {message:"Token_NotInitialized"},
  /**
   * insufficient balance
   */
  7: {message:"Token_InsufficientBalance"},
  /**
   * insufficient allowance
   */
  8: {message:"Token_InsufficientAllowance"},
  /**
   * amount must be positive
   */
  9: {message:"Token_InvalidAmount"},
  /**
   * Cross-contract call aborted
   */
  0: {message:"Aborted"},
  /**
   * Unknown error from cross-contract call
   */
  2147483647: {message:"UnknownError"}
}

export interface Client {
  /**
   * Construct and simulate a initialize transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize: ({admin, token_id}: {admin: string, token_id: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Deposit tokens into the vault.
   * The vault calls token.transfer_from, so the user must approve first.
   */
  deposit: ({user, amount}: {user: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Withdraw tokens from the vault back to the user.
   */
  withdraw: ({user, amount}: {user: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_deposit: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a total_deposited transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  total_deposited: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAABAAAAAAAAAAAAAAAClZhdWx0RXJyb3IAAAAAAAsAAAAZdmF1bHQgYWxyZWFkeSBpbml0aWFsaXplZAAAAAAAABJBbHJlYWR5SW5pdGlhbGl6ZWQAAAAAAAEAAAAVdmF1bHQgbm90IGluaXRpYWxpemVkAAAAAAAADk5vdEluaXRpYWxpemVkAAAAAAACAAAAH2RlcG9zaXQgYW1vdW50IG11c3QgYmUgcG9zaXRpdmUAAAAADUludmFsaWRBbW91bnQAAAAAAAADAAAAGndpdGhkcmF3YWwgZXhjZWVkcyBkZXBvc2l0AAAAAAATSW5zdWZmaWNpZW50RGVwb3NpdAAAAAAEAAAAE2FscmVhZHkgaW5pdGlhbGl6ZWQAAAAAGFRva2VuX0FscmVhZHlJbml0aWFsaXplZAAAAAUAAAAPbm90IGluaXRpYWxpemVkAAAAABRUb2tlbl9Ob3RJbml0aWFsaXplZAAAAAYAAAAUaW5zdWZmaWNpZW50IGJhbGFuY2UAAAAZVG9rZW5fSW5zdWZmaWNpZW50QmFsYW5jZQAAAAAAAAcAAAAWaW5zdWZmaWNpZW50IGFsbG93YW5jZQAAAAAAG1Rva2VuX0luc3VmZmljaWVudEFsbG93YW5jZQAAAAAIAAAAF2Ftb3VudCBtdXN0IGJlIHBvc2l0aXZlAAAAABNUb2tlbl9JbnZhbGlkQW1vdW50AAAAAAkAAAAbQ3Jvc3MtY29udHJhY3QgY2FsbCBhYm9ydGVkAAAAAAdBYm9ydGVkAAAAAAAAAAAmVW5rbm93biBlcnJvciBmcm9tIGNyb3NzLWNvbnRyYWN0IGNhbGwAAAAAAAxVbmtub3duRXJyb3J/////",
        "AAAAAAAAAAAAAAAKaW5pdGlhbGl6ZQAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAh0b2tlbl9pZAAAABMAAAABAAAD6QAAAAIAAAfQAAAAClZhdWx0RXJyb3IAAA==",
        "AAAAAAAAAGNEZXBvc2l0IHRva2VucyBpbnRvIHRoZSB2YXVsdC4KVGhlIHZhdWx0IGNhbGxzIHRva2VuLnRyYW5zZmVyX2Zyb20sIHNvIHRoZSB1c2VyIG11c3QgYXBwcm92ZSBmaXJzdC4AAAAAB2RlcG9zaXQAAAAAAgAAAAAAAAAEdXNlcgAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAAAgAAB9AAAAAKVmF1bHRFcnJvcgAA",
        "AAAAAAAAADBXaXRoZHJhdyB0b2tlbnMgZnJvbSB0aGUgdmF1bHQgYmFjayB0byB0aGUgdXNlci4AAAAId2l0aGRyYXcAAAACAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAACAAAH0AAAAApWYXVsdEVycm9yAAA=",
        "AAAAAAAAAAAAAAALZ2V0X2RlcG9zaXQAAAAAAQAAAAAAAAAEdXNlcgAAABMAAAABAAAACw==",
        "AAAAAAAAAAAAAAAPdG90YWxfZGVwb3NpdGVkAAAAAAAAAAABAAAACw==" ]),
      options
    )
  }
  public readonly fromJSON = {
    initialize: this.txFromJSON<Result<void>>,
        deposit: this.txFromJSON<Result<void>>,
        withdraw: this.txFromJSON<Result<void>>,
        get_deposit: this.txFromJSON<i128>,
        total_deposited: this.txFromJSON<i128>
  }
}