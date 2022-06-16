/**
 * Agent queries
 *
 * :TODO: wire into Personas hApp and replace generated agent names with serving of profile data
 *
 * @package: Holo-REA
 * @since:   2020-02-19
 */

import { DNAIdMappings, injectTypename, ReadParams } from '../types.js'
import { mapZomeFn, serializeHash, deserializeHash } from '../connection.js'

import {
  AccountingScope,
  Agent
} from '@valueflows/vf-graphql'
import { AgentPubKey } from '@holochain/client'
import { AgentResponse } from '../mutations/agent'

export interface RegistrationQueryParams {
  pubKey: AgentPubKey,
}

export default (dnaConfig: DNAIdMappings, conductorUri: string) => {

  //assumes there is a link from agentPubKey to a Person entry, but what if link cannot be resolved?
  const readMyAgent = mapZomeFn<null, Agent>(dnaConfig, conductorUri, 'agent', 'agent', 'get_my_agent')
  const readAgent = mapZomeFn<ReadParams, AgentResponse>(dnaConfig, conductorUri, 'agent', 'agent', 'get_agent')

  // read mapped DNA hash in order to construct VF-native IDs from DNA-local HC IDs
  const mappedDNA = dnaConfig['agent'] ? serializeHash(dnaConfig['agent'][0]) : null

  return {
    // :TODO: is myAgent always a 'Person' in Holochain, or will we allow users to act in an Organization context directly?
    myAgent: async (root, args): Promise<Agent> => {
      return readMyAgent(null)
    },

    // TODO: not totally sure on the significance of the `__typename` field injection, but once the `type` field is added to Agent, we could conditionally inject `Person` or `Organization`
    agent: injectTypename('Person', async (root, args): Promise<Agent> => {
      return (await readAgent({ address: args.id })).agent
    }),
  }
}
