import { Construct } from 'constructs';
import * as cdk from 'aws-cdk-lib';
import * as apigateway from "aws-cdk-lib/aws-apigateway";
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as path from 'path';
import * as awsLogs from 'aws-cdk-lib/aws-logs';
import { RestApiKey } from './RestApiKey';

export class ImageProcessingStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const getImageHandler = new lambda.Function(this, "GetImage", {
      runtime: lambda.Runtime.PROVIDED_AL2,
      code: lambda.Code.fromAsset(path.join(__dirname, "..", "functions/image-processing/target/lambda/get_image")),
      handler: "getimage.handler",
      logRetention: awsLogs.RetentionDays.FIVE_DAYS,
      memorySize: 128,
      tracing: lambda.Tracing.ACTIVE,
      timeout: cdk.Duration.minutes(3),
    });

    const postImageHandler = new lambda.Function(this, "PostImage", {
      runtime: lambda.Runtime.PROVIDED_AL2,
      code: lambda.Code.fromAsset(path.join(__dirname, "..", "functions/image-processing/target/lambda/post_image")),
      handler: "postimage.handler",
      logRetention: awsLogs.RetentionDays.FIVE_DAYS,
      memorySize: 128,
      tracing: lambda.Tracing.ACTIVE,
      timeout: cdk.Duration.minutes(3),
    });

    const api = new apigateway.RestApi(this, "Api", {
      restApiName: "ImageProcessing-Api",
      description: "ApiGateway for image processing handlers",
      binaryMediaTypes: ["*/*"]
    });

    // Require an api key to use the endpoints
    new RestApiKey(api);

    const apiEndpoint = api.root.addResource("api");
    const imageEndpoint = apiEndpoint.addResource("image");

    imageEndpoint.addMethod("GET", new apigateway.LambdaIntegration(getImageHandler), {
      apiKeyRequired: true
    });

    imageEndpoint.addMethod("POST", new apigateway.LambdaIntegration(postImageHandler), {
      apiKeyRequired: true
    });
  }
}
