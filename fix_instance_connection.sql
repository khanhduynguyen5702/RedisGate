-- Update first instance to use localhost instead of fake domain
UPDATE redis_instances
SET domain = '127.0.0.1',
    service_name = NULL
WHERE id = 'a9539b34-36d1-4da0-bc7b-00994c326655';

SELECT id, name, domain, service_name, port FROM redis_instances WHERE id = 'a9539b34-36d1-4da0-bc7b-00994c326655';

