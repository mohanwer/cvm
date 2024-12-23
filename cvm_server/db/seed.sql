
insert into applications
    (id, name, description)
values
    (
     '50b473ee-35b3-4252-8998-6be4d4130d3a',
     'infinite_hello',
     'prints hello with version'
    );

insert into application_versions
    (id, app_id, version, latest)
values
    (
     'd613a420-1b70-4a7e-8a05-518ad5da9c66',
     '50b473ee-35b3-4252-8998-6be4d4130d3a',
     '0.1.0',
     false
    );

insert into application_versions
    (id, app_id, version, latest)
values
    (
    'af8ba06c-3083-43e0-a5f0-0d60682736a4',
    '50b473ee-35b3-4252-8998-6be4d4130d3a',
    '0.2.0',
    true
    );

insert into application_builds
    (id, app_version_id, build_version, url, disabled)
values
    (
    'db559bec-db9e-4a1f-aeea-d75c409d4e2a',
    'd613a420-1b70-4a7e-8a05-518ad5da9c66',
    'x86_64-unknown-linux-gnu',
    'https://hello-versioned.s3.us-east-1.amazonaws.com/linux/infinite_hello_0.1.0',
    false
    );

insert into application_builds
    (id, app_version_id, build_version, url, disabled)
values
    (
    '76055cdf-202e-4d23-906c-b93f003bef27',
    'af8ba06c-3083-43e0-a5f0-0d60682736a4',
    'x86_64-unknown-linux-gnu',
    'https://hello-versioned.s3.us-east-1.amazonaws.com/linux/infinite_hello_0.2.0',
    false
    );

insert into application_builds
    (id, app_version_id, build_version, url, disabled)
values
    (
    '8617b74e-dcbc-41f3-b6f5-22b591002716',
    'd613a420-1b70-4a7e-8a05-518ad5da9c66',
    'x86_64-pc-windows-gnu',
    'https://hello-versioned.s3.us-east-1.amazonaws.com/windows/infinite_hello_0.1.0',
    false
    );

insert into application_builds
    (id, app_version_id, build_version, url, disabled)
values
    (
    '121c7d6e-3a10-4c36-8787-98eb6f0e3fee',
    'af8ba06c-3083-43e0-a5f0-0d60682736a4',
    'x86_64-pc-windows-gnu',
    'https://hello-versioned.s3.us-east-1.amazonaws.com/windows/infinite_hello_0.2.0',
    false
    );

insert into clients
    (id, app_id, build_version, version)
values
    (
    'd3da1ee8-bc09-44b0-9d5d-5f129c980c1d',
    '50b473ee-35b3-4252-8998-6be4d4130d3a',
    'x86_64-unknown-linux-gnu',
    '0.0.0'
    );

insert into clients
    (id, app_id, build_version, version)
values
    (
    'f944fc73-e1ca-442a-88bb-642d42a38a6c',
    '50b473ee-35b3-4252-8998-6be4d4130d3a',
    'x86_64-pc-windows-gnu',
    '0.0.0'
    );