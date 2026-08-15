package provider

import (
	"context"

	"github.com/hashicorp/terraform-plugin-framework/action"
	"github.com/hashicorp/terraform-plugin-framework/datasource"
	"github.com/hashicorp/terraform-plugin-framework/ephemeral"
	"github.com/hashicorp/terraform-plugin-framework/function"
	"github.com/hashicorp/terraform-plugin-framework/provider"
	"github.com/hashicorp/terraform-plugin-framework/provider/schema"
	"github.com/hashicorp/terraform-plugin-framework/resource"
)

var _ provider.Provider = &ContainerOsProvider{}
var _ provider.ProviderWithFunctions = &ContainerOsProvider{}
var _ provider.ProviderWithEphemeralResources = &ContainerOsProvider{}
var _ provider.ProviderWithActions = &ContainerOsProvider{}

type ContainerOsProvider struct {
	version string
}

func (p *ContainerOsProvider) Metadata(ctx context.Context, req provider.MetadataRequest, resp *provider.MetadataResponse) {
	resp.TypeName = "containeros"
	resp.Version = p.version
}

func (p *ContainerOsProvider) Schema(ctx context.Context, req provider.SchemaRequest, resp *provider.SchemaResponse) {
	resp.Schema = schema.Schema{}
}

func (p *ContainerOsProvider) Configure(ctx context.Context, req provider.ConfigureRequest, resp *provider.ConfigureResponse) {
}

func (p *ContainerOsProvider) Resources(ctx context.Context) []func() resource.Resource {
	return []func() resource.Resource{
		NewExampleResource,
	}
}

func (p *ContainerOsProvider) EphemeralResources(ctx context.Context) []func() ephemeral.EphemeralResource {
	return nil
}

func (p *ContainerOsProvider) DataSources(ctx context.Context) []func() datasource.DataSource {
	return nil
}

func (p *ContainerOsProvider) Functions(ctx context.Context) []func() function.Function {
	return nil
}

func (p *ContainerOsProvider) Actions(ctx context.Context) []func() action.Action {
	return nil
}

func New(version string) func() provider.Provider {
	return func() provider.Provider {
		return &ContainerOsProvider{
			version: version,
		}
	}
}
