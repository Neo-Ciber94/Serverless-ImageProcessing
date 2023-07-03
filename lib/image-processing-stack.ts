import { Construct } from 'constructs';
import * as cdk from 'aws-cdk-lib';
import * as apigateway from "aws-cdk-lib/aws-apigateway";
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as path from 'path';
import * as awsLogs from 'aws-cdk-lib/aws-logs';

export class ImageProcessingStack extends cdk.Stack {
  constructor(scope: Construct, id: string, apiKeys: string[], props?: cdk.StackProps) {
    super(scope, id, props);

    const getImageHandler = new lambda.Function(this, "GetImage", {
      runtime: lambda.Runtime.PROVIDED_AL2,
      code: lambda.Code.fromAsset(path.join(__dirname, "..", "functions/image-processing/target/lambda/get_image")),
      handler: "dummy.one",
      logRetention: awsLogs.RetentionDays.FIVE_DAYS,
      memorySize: 128,
      tracing: lambda.Tracing.ACTIVE,
      timeout: cdk.Duration.minutes(3),
    });

    const postImageHandler = new lambda.Function(this, "PostImage", {
      runtime: lambda.Runtime.PROVIDED_AL2,
      code: lambda.Code.fromAsset(path.join(__dirname, "..", "functions/image-processing/target/lambda/post_image")),
      handler: "dummy.two",
      logRetention: awsLogs.RetentionDays.FIVE_DAYS,
      memorySize: 128,
      tracing: lambda.Tracing.ACTIVE,
      timeout: cdk.Duration.minutes(3),

    });

    const restApi = new apigateway.RestApi(this, "Api", {
      restApiName: "ImageProcessing-Api",
      description: "ApiGateway for image processing handlers",
      binaryMediaTypes: ["*/*"]
    });

    const usagePlan = restApi.addUsagePlan("UsagePlan", {
      quota: {
        limit: 1000,
        period: apigateway.Period.DAY
      },
      throttle: {
        burstLimit: 10,
        rateLimit: 5
      },
    });

    if (apiKeys.length <= 0) {
      throw new Error("Expected at least 1 api key");
    }

    for (let i = 0; i < apiKeys.length; i++) {
      const apiKey = restApi.addApiKey(`DevApiKeyId-${i}`, {
        apiKeyName: `DevApiKey-${i}`,
        value: apiKeys[i]
      });

      usagePlan.addApiKey(apiKey);
    }

    const api = restApi.root.addResource("api");
    const imageEndpoint = api.addResource("image");

    imageEndpoint.addMethod("GET", new apigateway.LambdaIntegration(getImageHandler), {
      apiKeyRequired: true
    });

    imageEndpoint.addMethod("POST", new apigateway.LambdaIntegration(postImageHandler), {
      apiKeyRequired: true
    });
  }
}
