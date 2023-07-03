#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { ImageProcessingStack } from '../lib/image-processing-stack';
import { SSMClient, GetParameterCommand } from "@aws-sdk/client-ssm";

async function main() {
  const app = new cdk.App();

  const ssmClient = new SSMClient({
    region: process.env.AWS_DEFAULT_REGION
  });
  const apiKeysResult = await ssmClient.send(new GetParameterCommand({
    Name: "/image-handler/apikeys"
  }))

  const apiKeys = (apiKeysResult.Parameter?.Value || "")
    .split(",")
    .map(x => x.trim())
    .filter(x => x.length > 0);

  new ImageProcessingStack(app, 'ImageProcessingStack', apiKeys, {
    /* If you don't specify 'env', this stack will be environment-agnostic.
     * Account/Region-dependent features and context lookups will not work,
     * but a single synthesized template can be deployed anywhere. */

    /* Uncomment the next line to specialize this stack for the AWS Account
     * and Region that are implied by the current CLI configuration. */
    // env: { account: process.env.CDK_DEFAULT_ACCOUNT, region: process.env.CDK_DEFAULT_REGION },

    /* Uncomment the next line if you know exactly what Account and Region you
     * want to deploy the stack to. */
    // env: { account: '123456789012', region: 'us-east-1' },

    /* For more information, see https://docs.aws.amazon.com/cdk/latest/guide/environments.html */
  });
}

main().catch(console.error)
